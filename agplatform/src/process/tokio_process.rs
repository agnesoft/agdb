//! [internal]
//!
//! The asynchronous parts of [`ProcessImpl`](super::ProcessImpl) running
//! on [`tokio`].

use std::future::Future;
use std::pin::Pin;
use std::pin::pin;
use std::process::ExitStatus;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::task::Poll;
use std::time::Instant;

use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tokio::process::Child;

use super::EXIT_GRACE;
use super::OutputStream;
use super::Outputs;

const BUFFER_SIZE: usize = 64 * 1024;

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

/// Writes `input` to the standard input of `child` and reads its standard
/// output and error into `outputs` concurrently until they close and it
/// exits. Once it exited they are read at most for [`EXIT_GRACE`]. Past
/// `deadline`, once the sink of `outputs` fails or when cancelled it is
/// killed with its descendants if `group` (always on Windows), and `None`
/// is returned as the status unless cancelled.
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
    outputs: &Mutex<Outputs<'_>>,
) -> std::io::Result<Option<ExitStatus>> {
    let mut tree = Tree::new(child, group);
    let stdin = child.stdin.take();
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let mut communicated = None;

    let status = {
        let write = pin!(write(stdin, input));
        let read_stdout = pin!(read(stdout, OutputStream::Stdout, outputs));
        let read_stderr = pin!(read(stderr, OutputStream::Stderr, outputs));
        let mut communicate = pin!(join([write, read_stdout, read_stderr]));
        let mut wait = pin!(child.wait());
        let exit = std::future::poll_fn(|context| {
            if communicated.is_none()
                && let Poll::Ready(result) = communicate.as_mut().poll(context)
            {
                communicated = Some(result);
            }

            // WHY: the timer of `timeout_at` fires only once the runtime
            // ticks, after the child may have exited.
            if lock(outputs).failed() || deadline.is_some_and(|deadline| deadline <= Instant::now())
            {
                return Poll::Ready(None);
            }

            wait.as_mut().poll(context).map(Some)
        });

        let status = match deadline {
            Some(deadline) => tokio::time::timeout_at(deadline.into(), exit)
                .await
                .ok()
                .flatten(),
            None => exit.await,
        };

        communicated =
            collect_exit_grace(status.as_ref(), communicated, communicate.as_mut()).await;

        status
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
    let failure = communicated.and_then(Result::err);

    status.and_then(|status| failure.map_or(Ok(status), Err))
}

fn should_collect_exit_grace(
    status: Option<&std::io::Result<ExitStatus>>,
    communicated: Option<&std::io::Result<()>>,
) -> bool {
    status.is_some() && communicated.is_none()
}

async fn collect_exit_grace<F>(
    status: Option<&std::io::Result<ExitStatus>>,
    communicated: Option<std::io::Result<()>>,
    communicate: Pin<&mut F>,
) -> Option<std::io::Result<()>>
where
    F: Future<Output = std::io::Result<()>>,
{
    if should_collect_exit_grace(status, communicated.as_ref()) {
        tokio::time::timeout(EXIT_GRACE, communicate).await.ok()
    } else {
        communicated
    }
}

fn lock<'a, 'b>(outputs: &'a Mutex<Outputs<'b>>) -> std::sync::MutexGuard<'a, Outputs<'b>> {
    outputs.lock().unwrap_or_else(PoisonError::into_inner)
}

type Communication<'a> = Pin<&'a mut (dyn Future<Output = std::io::Result<()>> + Send)>;

/// Polls all `futures` until they complete and returns the first failure.
async fn join(mut futures: [Communication<'_>; 3]) -> std::io::Result<()> {
    let mut done = [false; 3];
    let mut failure = None;

    std::future::poll_fn(|context| {
        for (future, done) in futures.iter_mut().zip(&mut done) {
            if !*done && let Poll::Ready(result) = future.as_mut().poll(context) {
                *done = true;
                failure = failure.take().or(result.err());
            }
        }

        if done.iter().all(|done| *done) {
            Poll::Ready(failure.take().map_or(Ok(()), Err))
        } else {
            Poll::Pending
        }
    })
    .await
}

/// Reads `pipe` into `outputs` as its `stream` until it closes or fails,
/// or the sink of `outputs` fails.
async fn read<R: AsyncRead + Unpin>(
    pipe: Option<R>,
    stream: OutputStream,
    outputs: &Mutex<Outputs<'_>>,
) -> std::io::Result<()> {
    if let Some(mut pipe) = pipe {
        let mut buffer = vec![0; BUFFER_SIZE];

        loop {
            match pipe.read(&mut buffer).await? {
                0 => break,
                read => {
                    let mut outputs = lock(outputs);
                    outputs.push(stream, &buffer[..read]);

                    if outputs.failed() {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Writes `input` to `stdin` and closes it, stopping when the child closes
/// it, e.g. because it does not read it.
async fn write<W: AsyncWrite + Unpin>(stdin: Option<W>, input: &[u8]) -> std::io::Result<()> {
    match stdin {
        Some(mut stdin) => stdin.write_all(input).await.or_else(|error| {
            (error.kind() == std::io::ErrorKind::BrokenPipe)
                .then_some(())
                .ok_or(error)
        }),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            _context: &mut std::task::Context<'_>,
            buffer: &mut tokio::io::ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            Poll::Ready(
                self.0
                    .pop_front()
                    .unwrap_or(Ok(b""))
                    .map(|data| buffer.put_slice(data)),
            )
        }
    }

    mod join {
        use super::*;

        #[test]
        fn join_should_complete_all_and_return_first_failure() {
            let mut completed = 0;
            let ok = pin!(async { Ok(()) });
            let first = pin!(async { Err(std::io::Error::other("first")) });
            let second = pin!(async {
                completed += 1;
                Err(std::io::Error::other("second"))
            });

            let result = block_on(join([ok, first, second]));

            assert_eq!(result.unwrap_err().to_string(), "first");
            assert_eq!(completed, 1);
        }
    }

    mod should_collect_exit_grace {
        use super::*;

        #[test]
        fn should_collect_exit_grace_should_require_status_without_communication() {
            let ok_status = Some(Ok(status_ok()));
            let communication = Some(Ok(()));

            assert!(should_collect_exit_grace(ok_status.as_ref(), None));
            assert!(!should_collect_exit_grace(None, None));
            assert!(!should_collect_exit_grace(
                ok_status.as_ref(),
                communication.as_ref()
            ));
        }
    }

    mod collect_exit_grace {
        use super::*;

        #[test]
        fn collect_exit_grace_should_collect_only_when_needed() {
            let status = Ok(status_ok());
            let mut ready = std::future::ready(Ok(()));
            let collected = block_on(collect_exit_grace(
                Some(&status),
                None,
                Pin::new(&mut ready),
            ));
            assert!(collected.is_some());

            let mut ready = std::future::ready(Ok(()));
            let kept = block_on(collect_exit_grace(None, Some(Ok(())), Pin::new(&mut ready)));
            assert!(kept.unwrap().is_ok());
        }

        #[test]
        fn collect_exit_grace_timed_out_should_return_none() {
            let status = Ok(status_ok());
            let mut pending = std::future::pending::<std::io::Result<()>>();

            let collected = block_on(collect_exit_grace(
                Some(&status),
                None,
                Pin::new(&mut pending),
            ));

            assert!(collected.is_none());
        }
    }

    #[cfg(unix)]
    fn status_ok() -> ExitStatus {
        std::process::Command::new("sh")
            .args(["-c", "exit 0"])
            .status()
            .unwrap()
    }

    #[cfg(windows)]
    fn status_ok() -> ExitStatus {
        std::process::Command::new("cmd")
            .args(["/C", "exit 0"])
            .status()
            .unwrap()
    }

    mod read {
        use super::*;

        /// Reads `pipe` as the standard output of a run passing it to
        /// `sink`, if any, and returns the result with the output captured.
        fn read_run(
            pipe: Option<Scripted>,
            sink: Option<&mut crate::process::OutputSink<'_>>,
        ) -> (std::io::Result<()>, Vec<u8>) {
            let (mut sink, mut captured) = (sink, Vec::new());
            let outputs = Mutex::new(Outputs::new(|stream, data| match sink.as_mut() {
                Some(sink) => sink(stream, data),
                None => {
                    captured.extend_from_slice(data);
                    Ok(())
                }
            }));
            let result = block_on(read(pipe, OutputStream::Stdout, &outputs));
            drop(outputs);
            (result, captured)
        }

        #[test]
        fn read_should_capture_data_until_end() {
            let (result, output) =
                read_run(Some(Scripted([Ok(&b"a"[..]), Ok(&b"b"[..])].into())), None);

            assert!(result.is_ok());
            assert_eq!(output, b"ab");
        }

        #[test]
        fn read_without_pipe_should_capture_nothing() {
            let (result, output) = read_run(None, None);

            assert!(result.is_ok());
            assert!(output.is_empty());
        }

        #[test]
        fn read_failing_should_fail() {
            let (result, _) = read_run(
                Some(Scripted([Err(std::io::Error::other("failed"))].into())),
                None,
            );

            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::Other);
        }

        #[test]
        fn read_failing_sink_should_stop_reading() {
            let mut calls = 0;
            let mut sink = |_: OutputStream, _: &[u8]| {
                calls += 1;
                Err(crate::Error::process("full"))
            };

            let (result, _) = read_run(
                Some(Scripted([Ok(&b"a"[..]), Ok(&b"b"[..])].into())),
                Some(&mut sink),
            );

            assert!(result.is_ok());
            assert_eq!(calls, 1);
        }
    }

    mod write {
        use super::*;

        #[test]
        fn write_to_closed_pipe_should_succeed() {
            let (writer, reader) = tokio::io::duplex(1);
            drop(reader);

            let result = block_on(write(Some(writer), b"input"));

            assert!(result.is_ok());
        }

        #[test]
        fn write_failing_should_fail() {
            let mut full = [0; 0];

            let error =
                block_on(write(Some(std::io::Cursor::new(&mut full[..])), b"input")).unwrap_err();

            assert_eq!(error.kind(), std::io::ErrorKind::WriteZero);
        }
    }
}
