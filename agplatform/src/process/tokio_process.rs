//! [internal]
//!
//! The asynchronous parts of [`ProcessImpl`](super::ProcessImpl) running
//! on [`tokio`].

use std::future::Future;
use std::pin::Pin;
use std::pin::pin;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::task::Context;
use std::task::Poll;
use std::time::Duration;
use std::time::Instant;

use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::ReadBuf;
use tokio::process::Child;

use crate::Result;
use crate::interceptor::BoxFuture;
use crate::interceptor::Emit;

use super::DRAIN_LIMIT;
use super::Ended;
use super::OutputStream;
use super::ProcessCommand;
use super::grace_end;

const BUFFER_SIZE: usize = 64 * 1024;

const STREAMS: [OutputStream; 2] = [OutputStream::Stdout, OutputStream::Stderr];

type Input<'a> = Box<dyn AsyncWrite + Send + Unpin + 'a>;

type Output<'a> = Box<dyn AsyncRead + Send + Unpin + 'a>;

/// The first failure to use the pipes of a child, the result of passing
/// its output on and whether its output was left open, see [`Ended`].
type Communicated = (std::io::Result<()>, Result, bool);

/// [internal]
///
/// Passes the chunks of the output of a child process on, see
/// [`Emit::emit`].
pub(super) trait Pass: Send {
    /// Passes the `data` of `stream` on.
    fn pass<'a>(&'a mut self, stream: OutputStream, data: &'a [u8]) -> BoxFuture<'a, Result>;
}

#[cfg(unix)]
type TreeHandle = u32;
#[cfg(windows)]
type TreeHandle = std::os::windows::io::OwnedHandle;

/// The process tree of a child, killed when dropped unless disarmed, e.g.
/// when the run is cancelled.
struct Tree(Option<TreeHandle>);

impl Tree {
    /// The process group of `child` if it leads one, see
    /// [`configure`](super::unix::configure).
    #[cfg(unix)]
    fn new(child: &Child, group: bool) -> Self {
        Self(child.id().filter(|_| group))
    }

    /// A new job holding `child`, see [`job`](super::windows::job).
    #[cfg(windows)]
    fn new(child: &Child, _group: bool) -> Self {
        Self(child.raw_handle().and_then(super::windows::job))
    }

    fn kill(&mut self) {
        if let Some(tree) = self.0.take() {
            #[cfg(unix)]
            super::unix::kill_group(tree);
            #[cfg(windows)]
            super::windows::terminate(&tree);
        }
    }

    /// Lets the tree outlive the run, see [`release`](super::windows::release).
    fn disarm(&mut self) {
        #[cfg(windows)]
        if let Some(job) = &self.0 {
            super::windows::release(job);
        }

        self.0 = None;
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        self.kill();
    }
}

impl Pass for Emit<'_, ProcessCommand> {
    fn pass<'a>(&'a mut self, stream: OutputStream, data: &'a [u8]) -> BoxFuture<'a, Result> {
        Box::pin(self.emit(stream, data))
    }
}

/// Writes `input` to the standard input of `child` and reads its standard
/// output and error concurrently, passing each chunk on with `pass`, until
/// they close and it exits. Once it exited they are read on for at most
/// `grace`, see [`communicate`]. Past `deadline` or once passing a chunk on
/// fails while it runs, or when cancelled, it is killed with its
/// descendants if `group` (always on Windows), otherwise alone, and `None`
/// is returned as the status unless cancelled. Also returns the result of
/// passing the output on.
///
/// # Errors
///
/// Returns the first failure to use the pipes, once the child exited, as
/// its output would be incomplete.
pub(super) async fn wait_with_output(
    child: &mut Child,
    group: bool,
    input: &[u8],
    deadline: Option<Instant>,
    grace: Duration,
    pass: &mut dyn Pass,
) -> (std::io::Result<Ended>, Result) {
    let mut tree = Tree::new(child, group);
    let stdin = child.stdin.take().map(|pipe| Box::new(pipe) as Input);
    let stdout = child.stdout.take().map(|pipe| Box::new(pipe) as Output);
    let stderr = child.stderr.take().map(|pipe| Box::new(pipe) as Output);
    let exited = AtomicBool::new(false);
    let mut communicated = None;
    let mut exit = None;

    let status = {
        let mut communicate = pin!(communicate(
            stdin,
            input,
            [stdout, stderr],
            &exited,
            grace,
            pass
        ));
        let mut wait = pin!(child.wait());
        let mut expiry = pin!(deadline.map(|deadline| tokio::time::sleep_until(deadline.into())));

        std::future::poll_fn(|context| {
            loop {
                if communicated.is_none()
                    && let Poll::Ready(result) = communicate.as_mut().poll(context)
                {
                    communicated = Some(result);
                }

                if exit.is_some() {
                    return match communicated {
                        Some(_) => Poll::Ready(exit.take()),
                        None => Poll::Pending,
                    };
                }

                // WHY: the timer fires only once the runtime ticks, after
                // the child may have exited.
                if communicated
                    .as_ref()
                    .is_some_and(|(_, passed, _): &Communicated| passed.is_err())
                    || deadline.is_some_and(|deadline| deadline <= Instant::now())
                    || expiry
                        .as_mut()
                        .as_pin_mut()
                        .is_some_and(|expiry| expiry.poll(context).is_ready())
                {
                    return Poll::Ready(None);
                }

                let Poll::Ready(status) = wait.as_mut().poll(context) else {
                    return Poll::Pending;
                };
                exit = Some(status);
                exited.store(true, Ordering::Relaxed);
            }
        })
        .await
    };

    let status = match status {
        Some(status) => status.map(Some),
        None => {
            tree.kill();
            let _ = child.start_kill();
            child.wait().await.map(|_| None)
        }
    };
    tree.disarm();
    let (failure, passed, truncated) = communicated
        .map_or((None, Ok(()), false), |(used, passed, truncated)| {
            (used.err(), passed, truncated)
        });

    (
        status.and_then(|status| failure.map_or(Ok(Ended { status, truncated }), Err)),
        passed,
    )
}

/// Writes `input` to `stdin` and reads the standard output and error
/// `outputs` concurrently until they close, passing each chunk on with
/// `pass`. A chunk is awaited before reading on, so that a sink applies
/// backpressure. Ends once passing a chunk on fails, or `grace` after it
/// sees that the child `exited` between chunks, as its descendants may
/// hold the pipes open, once nothing is ready to be read or [`DRAIN_LIMIT`]
/// more was. Returns whether the outputs were left open then.
async fn communicate(
    mut stdin: Option<Input<'_>>,
    mut input: &[u8],
    mut outputs: [Option<Output<'_>>; 2],
    exited: &AtomicBool,
    grace: Duration,
    pass: &mut dyn Pass,
) -> Communicated {
    let mut buffers = outputs
        .each_ref()
        .map(|pipe| vec![0; pipe.as_ref().map_or(0, |_| BUFFER_SIZE)]);
    let mut failure = None;
    let mut first = 0;
    let mut grace_timer = None::<Option<Pin<Box<tokio::time::Sleep>>>>;
    let mut drained = 0;

    loop {
        let read = std::future::poll_fn(|context| {
            failure = failure
                .take()
                .or(poll_write(&mut stdin, &mut input, context).err());

            let ended = exited.load(Ordering::Relaxed)
                && grace_timer
                    .get_or_insert_with(|| {
                        grace_end(grace)
                            .map(|deadline| Box::pin(tokio::time::sleep_until(deadline.into())))
                    })
                    .as_mut()
                    .is_some_and(|timer| timer.as_mut().poll(context).is_ready());

            // WHY: what is ready is read even once the grace ended, as a
            // busy runtime may see the output ready only as it sees the
            // grace end, long after the child exited.
            // WHY: they take turns to be read first, so that a busy pipe
            // does not starve the other.
            for index in [first, 1 - first]
                .into_iter()
                .filter(|_| !ended || drained < DRAIN_LIMIT)
            {
                match poll_read(&mut outputs[index], &mut buffers[index], context) {
                    Some(Ok(read)) => return Poll::Ready(Some((index, read, ended))),
                    Some(Err(error)) => failure = failure.take().or(Some(error)),
                    None => {}
                }
            }

            if ended || (stdin.is_none() && outputs.iter().all(Option::is_none)) {
                Poll::Ready(None)
            } else {
                Poll::Pending
            }
        })
        .await;

        let Some((index, read, ended)) = read else {
            break (
                failure.map_or(Ok(()), Err),
                Ok(()),
                outputs.iter().any(Option::is_some),
            );
        };
        first = 1 - index;

        if ended {
            drained += read;
        }

        if let Err(error) = pass.pass(STREAMS[index], &buffers[index][..read]).await {
            break (failure.map_or(Ok(()), Err), Err(error), false);
        }
    }
}

/// Reads what is available from `pipe` into `buffer`, returning the bytes
/// read, if any, closing it at its end or when it fails.
fn poll_read(
    pipe: &mut Option<Output<'_>>,
    buffer: &mut [u8],
    context: &mut Context<'_>,
) -> Option<std::io::Result<usize>> {
    let mut buffer = ReadBuf::new(buffer);

    match Pin::new(pipe.as_mut()?).poll_read(context, &mut buffer) {
        Poll::Pending => None,
        Poll::Ready(Ok(())) if !buffer.filled().is_empty() => Some(Ok(buffer.filled().len())),
        Poll::Ready(result) => {
            *pipe = None;
            result.err().map(Err)
        }
    }
}

/// Writes what `stdin` takes of `input` and leaves the rest in it, closing
/// `stdin` once it is all written, the child closed it, e.g. because it
/// does not read it, or it fails.
fn poll_write(
    stdin: &mut Option<Input<'_>>,
    input: &mut &[u8],
    context: &mut Context<'_>,
) -> std::io::Result<()> {
    let result = loop {
        let Some(pipe) = stdin.as_mut().filter(|_| !input.is_empty()) else {
            break Ok(());
        };

        match Pin::new(pipe).poll_write(context, input) {
            Poll::Pending => return Ok(()),
            Poll::Ready(Ok(written @ 1..)) => *input = &input[written..],
            Poll::Ready(result) => {
                break result
                    .and(Err(std::io::ErrorKind::WriteZero.into()))
                    .or_else(|error| {
                        (error.kind() == std::io::ErrorKind::BrokenPipe)
                            .then_some(())
                            .ok_or(error)
                    });
            }
        }
    };
    *stdin = None;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::EXIT_GRACE;
    use tokio::io::AsyncReadExt;

    /// Runs `future` to completion on a new runtime.
    fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap()
            .block_on(future)
    }

    /// A pipe returning the scripted `reads` in order, each data or a
    /// failure, and then its end.
    struct Scripted(std::collections::VecDeque<std::io::Result<&'static [u8]>>);

    impl AsyncRead for Scripted {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _context: &mut Context<'_>,
            buffer: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(
                self.0
                    .pop_front()
                    .unwrap_or(Ok(b""))
                    .map(|data| buffer.put_slice(data)),
            )
        }
    }

    fn scripted<const N: usize>(
        reads: [std::io::Result<&'static [u8]>; N],
    ) -> Option<Output<'static>> {
        Some(Box::new(Scripted(reads.into())))
    }

    /// Records the chunks passed on after waiting for the runtime once,
    /// failing each if `fail`.
    #[derive(Default)]
    struct Recorder {
        chunks: Vec<(OutputStream, Vec<u8>)>,
        fail: bool,
    }

    impl Pass for Recorder {
        fn pass<'a>(&'a mut self, stream: OutputStream, data: &'a [u8]) -> BoxFuture<'a, Result> {
            Box::pin(async move {
                tokio::task::yield_now().await;
                self.chunks.push((stream, data.to_vec()));

                if self.fail {
                    Err(crate::Error::process("full"))
                } else {
                    Ok(())
                }
            })
        }
    }

    fn context() -> Context<'static> {
        Context::from_waker(std::task::Waker::noop())
    }

    mod communicate {
        use super::*;

        #[test]
        fn communicate_should_pass_chunks_on_in_order_taking_turns() {
            // GIVEN two chunks of the standard output and one of the error
            let mut recorder = Recorder::default();
            let outputs = [
                scripted([Ok(&b"a"[..]), Ok(&b"b"[..])]),
                scripted([Ok(&b"c"[..])]),
            ];

            // WHEN communicating with a sink waiting for each chunk
            let (used, passed, truncated) = block_on(communicate(
                None,
                b"",
                outputs,
                &AtomicBool::new(false),
                EXIT_GRACE,
                &mut recorder,
            ));

            // THEN the streams take turns, each in order
            assert!(used.is_ok());
            assert!(passed.is_ok());
            assert!(!truncated);
            assert_eq!(
                recorder.chunks,
                [
                    (OutputStream::Stdout, b"a".to_vec()),
                    (OutputStream::Stderr, b"c".to_vec()),
                    (OutputStream::Stdout, b"b".to_vec()),
                ]
            );
        }

        #[test]
        fn communicate_should_write_input_and_close_stdin() {
            let (writer, mut reader) = tokio::io::duplex(64);
            let mut written = Vec::new();

            let (used, _, _) = block_on(async {
                let communicated = communicate(
                    Some(Box::new(writer)),
                    b"input",
                    [None, None],
                    &AtomicBool::new(false),
                    EXIT_GRACE,
                    &mut Recorder::default(),
                )
                .await;
                reader.read_to_end(&mut written).await.unwrap();
                communicated
            });

            assert!(used.is_ok());
            assert_eq!(written, b"input");
        }

        #[test]
        fn communicate_failing_pass_should_end_with_its_error() {
            let mut recorder = Recorder {
                fail: true,
                ..Default::default()
            };

            let (used, passed, truncated) = block_on(communicate(
                None,
                b"",
                [scripted([Ok(&b"a"[..]), Ok(&b"b"[..])]), None],
                &AtomicBool::new(false),
                EXIT_GRACE,
                &mut recorder,
            ));

            assert!(used.is_ok());
            assert_eq!(passed.unwrap_err().description(), "full");
            assert!(!truncated);
            assert_eq!(recorder.chunks.len(), 1);
        }

        #[test]
        fn communicate_exited_child_should_read_pipes_held_open_for_grace() {
            // GIVEN a chunk in a pipe held open after the child exited
            let (mut writer, reader) = tokio::io::duplex(8);
            let mut recorder = Recorder::default();

            // WHEN communicating
            let (used, passed, truncated) = block_on(async {
                tokio::io::AsyncWriteExt::write_all(&mut writer, b"a")
                    .await
                    .unwrap();
                communicate(
                    None,
                    b"",
                    [Some(Box::new(reader)), None],
                    &AtomicBool::new(true),
                    EXIT_GRACE,
                    &mut recorder,
                )
                .await
            });

            // THEN the chunk is passed on and the pipe left open once the
            // grace ends
            assert!(used.is_ok());
            assert!(passed.is_ok());
            assert!(truncated);
            assert_eq!(recorder.chunks, [(OutputStream::Stdout, b"a".to_vec())]);
        }

        #[test]
        fn communicate_starved_past_grace_should_read_output_ready() {
            // GIVEN an exited child whose output becomes ready while the
            // runtime is blocked past the grace
            let (mut writer, reader) = tokio::io::duplex(8);
            let grace = Duration::from_millis(10);
            let mut recorder = Recorder::default();

            // WHEN communicating
            let (used, passed, truncated) = block_on(async {
                let writing = tokio::spawn(async move {
                    std::thread::sleep(10 * grace);
                    tokio::io::AsyncWriteExt::write_all(&mut writer, b"a")
                        .await
                        .unwrap();
                });
                let communicated = communicate(
                    None,
                    b"",
                    [Some(Box::new(reader)), None],
                    &AtomicBool::new(true),
                    grace,
                    &mut recorder,
                )
                .await;
                writing.await.unwrap();
                communicated
            });

            // THEN all of it is passed on up to the end of the pipe
            assert!(used.is_ok());
            assert!(passed.is_ok());
            assert!(!truncated);
            assert_eq!(recorder.chunks, [(OutputStream::Stdout, b"a".to_vec())]);
        }

        #[test]
        fn communicate_grace_ended_endless_output_should_stop_at_drain_limit() {
            // GIVEN a pipe that is always ready, e.g. written by a descendant
            // of an exited child
            struct Endless;

            impl AsyncRead for Endless {
                fn poll_read(
                    self: Pin<&mut Self>,
                    _context: &mut Context<'_>,
                    buffer: &mut ReadBuf<'_>,
                ) -> Poll<std::io::Result<()>> {
                    buffer.put_slice(&vec![b'y'; buffer.remaining()]);
                    Poll::Ready(Ok(()))
                }
            }

            let mut recorder = Recorder::default();

            // WHEN communicating once its grace ended
            let (used, passed, truncated) = block_on(communicate(
                None,
                b"",
                [Some(Box::new(Endless)), None],
                &AtomicBool::new(true),
                Duration::ZERO,
                &mut recorder,
            ));

            // THEN it is read on up to the limit, also before the timer of
            // the grace fires, and left open
            assert!(used.is_ok());
            assert!(passed.is_ok());
            assert!(truncated);
            assert!(
                recorder
                    .chunks
                    .iter()
                    .map(|(_, data)| data.len())
                    .sum::<usize>()
                    >= DRAIN_LIMIT
            );
        }

        #[test]
        fn communicate_failing_pipe_should_read_others_and_fail() {
            let mut recorder = Recorder::default();

            let (used, passed, truncated) = block_on(communicate(
                None,
                b"",
                [
                    scripted([Err(std::io::Error::other("failed"))]),
                    scripted([Ok(&b"c"[..])]),
                ],
                &AtomicBool::new(false),
                EXIT_GRACE,
                &mut recorder,
            ));

            assert_eq!(used.unwrap_err().kind(), std::io::ErrorKind::Other);
            assert!(passed.is_ok());
            assert!(!truncated);
            assert_eq!(recorder.chunks, [(OutputStream::Stderr, b"c".to_vec())]);
        }
    }

    mod poll_read {
        use super::*;

        #[test]
        fn poll_read_should_return_bytes_read() {
            let mut pipe = scripted([Ok(&b"ab"[..])]);
            let mut buffer = [0; 8];

            let read = poll_read(&mut pipe, &mut buffer, &mut context());

            assert_eq!(read.unwrap().unwrap(), 2);
            assert_eq!(&buffer[..2], b"ab");
            assert!(pipe.is_some());
        }

        #[test]
        fn poll_read_at_end_should_close_pipe() {
            let mut pipe = scripted([]);

            let read = poll_read(&mut pipe, &mut [0; 8], &mut context());

            assert!(read.is_none());
            assert!(pipe.is_none());
        }

        #[test]
        fn poll_read_failing_should_close_pipe_and_fail() {
            let mut pipe = scripted([Err(std::io::Error::other("failed"))]);

            let read = poll_read(&mut pipe, &mut [0; 8], &mut context());

            assert_eq!(read.unwrap().unwrap_err().kind(), std::io::ErrorKind::Other);
            assert!(pipe.is_none());
        }

        #[test]
        fn poll_read_with_nothing_to_read_yet_should_keep_pipe() {
            let (_writer, reader) = tokio::io::duplex(8);
            let mut pipe: Option<Output> = Some(Box::new(reader));

            let read = poll_read(&mut pipe, &mut [0; 8], &mut context());

            assert!(read.is_none());
            assert!(pipe.is_some());
        }
    }

    mod poll_write {
        use super::*;

        #[test]
        fn poll_write_to_full_pipe_should_keep_rest_of_input() {
            let (writer, _reader) = tokio::io::duplex(1);
            let mut stdin: Option<Input> = Some(Box::new(writer));
            let mut input: &[u8] = b"input";

            let result = poll_write(&mut stdin, &mut input, &mut context());

            assert!(result.is_ok());
            assert_eq!(input, b"nput");
            assert!(stdin.is_some());
        }

        #[test]
        fn poll_write_to_closed_pipe_should_close_stdin() {
            let (writer, reader) = tokio::io::duplex(1);
            drop(reader);
            let mut stdin: Option<Input> = Some(Box::new(writer));

            let result = poll_write(&mut stdin, &mut &b"input"[..], &mut context());

            assert!(result.is_ok());
            assert!(stdin.is_none());
        }

        #[test]
        fn poll_write_failing_should_close_stdin_and_fail() {
            let mut full = [0; 0];
            let mut stdin: Option<Input> = Some(Box::new(std::io::Cursor::new(&mut full[..])));

            let error = poll_write(&mut stdin, &mut &b"input"[..], &mut context()).unwrap_err();

            assert_eq!(error.kind(), std::io::ErrorKind::WriteZero);
            assert!(stdin.is_none());
        }
    }
}
