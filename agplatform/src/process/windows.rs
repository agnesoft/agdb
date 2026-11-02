//! [internal]
//!
//! The Windows specific parts of [`ProcessImpl`](super::ProcessImpl).

use std::ffi::c_void;
use std::os::windows::io::AsRawHandle;
use std::os::windows::io::FromRawHandle;
use std::os::windows::io::OwnedHandle;
use std::os::windows::io::RawHandle;
use std::process::Child;
use std::process::Command;
use std::process::ExitStatus;
use std::time::Instant;

use super::EXIT_GRACE;
use super::OutputStream;
use super::Outputs;
use super::millis_until;

const BUFFER_SIZE: usize = 8192;
const ERROR_BROKEN_PIPE: i32 = 109;
const ERROR_HANDLE_EOF: i32 = 38;
const ERROR_IO_PENDING: i32 = 997;
const ERROR_NO_DATA: i32 = 232;
const INFINITE: u32 = u32::MAX;
const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION: i32 = 9;
const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x2000;
const PROCESS: usize = 3;
const WAIT_FAILED: u32 = u32::MAX;

/// `JOBOBJECT_EXTENDED_LIMIT_INFORMATION`, of which only the limit flags
/// are set.
#[repr(C)]
#[derive(Default)]
struct ExtendedLimitInformation {
    _per_process_user_time_limit: i64,
    _per_job_user_time_limit: i64,
    limit_flags: u32,
    _minimum_working_set_size: usize,
    _maximum_working_set_size: usize,
    _active_process_limit: u32,
    _affinity: usize,
    _priority_class: u32,
    _scheduling_class: u32,
    _io_counters: [u64; 6],
    _process_memory_limit: usize,
    _job_memory_limit: usize,
    _peak_process_memory_used: usize,
    _peak_job_memory_used: usize,
}

#[repr(C)]
struct Overlapped {
    internal: usize,
    internal_high: usize,
    offset: u32,
    offset_high: u32,
    event: RawHandle,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn AssignProcessToJobObject(job: RawHandle, process: RawHandle) -> i32;

    fn CancelIoEx(file: RawHandle, overlapped: *mut Overlapped) -> i32;

    fn CreateEventW(
        attributes: *const c_void,
        manual_reset: i32,
        initial_state: i32,
        name: *const u16,
    ) -> RawHandle;

    fn CreateJobObjectW(attributes: *const c_void, name: *const u16) -> RawHandle;

    fn GetOverlappedResult(
        file: RawHandle,
        overlapped: *mut Overlapped,
        transferred: *mut u32,
        wait: i32,
    ) -> i32;

    fn SetInformationJobObject(
        job: RawHandle,
        class: i32,
        information: *const c_void,
        length: u32,
    ) -> i32;

    fn ReadFile(
        file: RawHandle,
        buffer: *mut u8,
        length: u32,
        read: *mut u32,
        overlapped: *mut Overlapped,
    ) -> i32;

    fn TerminateJobObject(job: RawHandle, exit_code: u32) -> i32;

    fn WaitForMultipleObjects(
        count: u32,
        handles: *const RawHandle,
        wait_all: i32,
        milliseconds: u32,
    ) -> u32;

    fn WriteFile(
        file: RawHandle,
        buffer: *const u8,
        length: u32,
        written: *mut u32,
        overlapped: *mut Overlapped,
    ) -> i32;
}

/// A pipe with at most one overlapped operation in flight, signalling its
/// event when the operation completes.
///
/// WHY: std creates its ends of the child pipes for overlapped I/O, which
/// is what its own `Command::output` relies on. It is not a documented
/// guarantee: if it changed, the reads would block one at a time, so
/// re-check `std::sys::pal::windows::pipe::anon_pipe` on toolchain
/// upgrades.
struct Channel {
    pipe: Option<OwnedHandle>,
    event: OwnedHandle,
    // WHY: boxed with the buffer so that both stay in place while an
    // operation is in flight, even if the channel moves.
    overlapped: Box<Overlapped>,
    buffer: Vec<u8>,
    pending: bool,
}

impl Channel {
    fn new(pipe: Option<OwnedHandle>) -> std::io::Result<Self> {
        // SAFETY: no attributes and no name, a manual-reset event that is
        // initially not signalled.
        let event = unsafe { CreateEventW(std::ptr::null(), 1, 0, std::ptr::null()) };

        (!event.is_null())
            .then(|| Self {
                pipe,
                // SAFETY: `event` is a new handle owned by nothing else.
                event: unsafe { OwnedHandle::from_raw_handle(event) },
                overlapped: Box::new(Overlapped {
                    internal: 0,
                    internal_high: 0,
                    offset: 0,
                    offset_high: 0,
                    event,
                }),
                buffer: vec![0; BUFFER_SIZE],
                pending: false,
            })
            .ok_or_else(std::io::Error::last_os_error)
    }

    fn is_idle(&self) -> bool {
        self.pipe.is_some() && !self.pending
    }

    fn handle(&self) -> RawHandle {
        self.pipe
            .as_ref()
            .map_or(std::ptr::null_mut(), AsRawHandle::as_raw_handle)
    }

    /// Completes the operation in flight, returning the bytes transferred
    /// or `None` and closing the pipe if it failed, e.g. because it was
    /// closed by the child.
    ///
    /// # Errors
    ///
    /// Returns the failure of the operation unless the pipe was closed.
    fn finish(&mut self) -> std::io::Result<Option<usize>> {
        let mut transferred = 0;
        self.pending = false;

        // SAFETY: the operation of `overlapped` on the pipe has completed as
        // its event is signalled.
        let result = unsafe {
            GetOverlappedResult(self.handle(), &mut *self.overlapped, &mut transferred, 0)
        };

        if result != 0 {
            return Ok(Some(transferred as usize));
        }

        let error = std::io::Error::last_os_error();
        self.pipe = None;
        closed(error).map(|()| None)
    }

    fn start_read(&mut self) -> std::io::Result<()> {
        // SAFETY: the buffer and `overlapped` stay in place and alive until
        // the operation completes, see `Drop`.
        let result = unsafe {
            ReadFile(
                self.handle(),
                self.buffer.as_mut_ptr(),
                self.buffer.len() as u32,
                std::ptr::null_mut(),
                &mut *self.overlapped,
            )
        };
        self.started(result)
    }

    fn start_write(&mut self, input: &[u8]) -> std::io::Result<()> {
        // SAFETY: `input` and `overlapped` stay in place and alive until
        // the operation completes, see `Drop` and `wait_with_output`.
        let result = unsafe {
            WriteFile(
                self.handle(),
                input.as_ptr(),
                input.len().min(u32::MAX as usize) as u32,
                std::ptr::null_mut(),
                &mut *self.overlapped,
            )
        };
        self.started(result)
    }

    fn started(&mut self, result: i32) -> std::io::Result<()> {
        let error = std::io::Error::last_os_error();

        // WHY: an operation that completes right away signals the event as
        // well, so it is finished like a pending one.
        if result != 0 || error.raw_os_error() == Some(ERROR_IO_PENDING) {
            self.pending = true;
            Ok(())
        } else {
            self.pipe = None;
            closed(error)
        }
    }
}

/// Succeeds for the `error` of a pipe closed by the child, fails with it
/// otherwise.
fn closed(error: std::io::Error) -> std::io::Result<()> {
    match error.raw_os_error() {
        Some(ERROR_BROKEN_PIPE | ERROR_HANDLE_EOF | ERROR_NO_DATA) => Ok(()),
        _ => Err(error),
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        if self.pending {
            let mut transferred = 0;

            // SAFETY: cancels the operation in flight and waits for it to
            // complete, so that its buffer and `overlapped` are no longer in
            // use when freed.
            unsafe {
                CancelIoEx(self.handle(), &mut *self.overlapped);
                GetOverlappedResult(self.handle(), &mut *self.overlapped, &mut transferred, 1);
            }
        }
    }
}

pub(super) fn configure(_command: &mut Command, _group: bool) {}

/// Writes `input` to the standard input of `child` and reads its standard
/// output and error into `outputs` on the current thread until they close
/// and it exits. Once it exited they are read at most for [`EXIT_GRACE`].
/// Past `deadline`, or once the sink of `outputs` fails, it is killed with
/// its descendants and `None` is returned as the status.
///
/// # Errors
///
/// Returns the first failure to use the pipes, once the child exited, as
/// its output would be incomplete. A failed pipe is closed, all of them if
/// they cannot be waited for.
pub(super) fn wait_with_output(
    child: &mut Child,
    input: &[u8],
    deadline: Option<Instant>,
    outputs: &mut Outputs,
) -> std::io::Result<Option<ExitStatus>> {
    let job = job(child.as_raw_handle());
    let stdin = child.stdin.take().map(OwnedHandle::from);
    let stdout = child.stdout.take().map(OwnedHandle::from);
    let stderr = child.stderr.take().map(OwnedHandle::from);

    Channel::new(stdin).and_then(|stdin| {
        Channel::new(stdout).and_then(|stdout| {
            Channel::new(stderr).and_then(|stderr| {
                communicate(
                    child,
                    job.as_ref(),
                    [stdin, stdout, stderr],
                    input,
                    deadline,
                    outputs,
                )
            })
        })
    })
}

/// Returns a new job holding the child `process`, so that the processes
/// it starts can be killed with it even after it exited, see
/// [`terminate`], or `None` if it cannot be created. They are killed as
/// well when the job is closed, e.g. when the current process dies, unless
/// [`release`]d before.
///
/// WHY: the processes the child starts before being assigned escape the
/// job, which std leaves no way to prevent as it cannot spawn suspended.
pub(super) fn job(process: RawHandle) -> Option<OwnedHandle> {
    // SAFETY: no attributes and no name.
    let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };

    (!job.is_null())
        // SAFETY: `job` is a new handle owned by nothing else.
        .then(|| unsafe { OwnedHandle::from_raw_handle(job) })
        .filter(|job| {
            // SAFETY: both are open handles.
            limit(job, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE)
                && unsafe { AssignProcessToJobObject(job.as_raw_handle(), process) != 0 }
        })
}

/// Lets the processes of `job` outlive it once closed, see [`job`].
pub(super) fn release(job: &OwnedHandle) {
    limit(job, 0);
}

/// Sets the limit `flags` of `job`, returning whether it succeeded.
fn limit(job: &OwnedHandle, flags: u32) -> bool {
    let information = ExtendedLimitInformation {
        limit_flags: flags,
        ..Default::default()
    };

    // SAFETY: `information` is the structure of the class, alive for the
    // duration of the call.
    unsafe {
        SetInformationJobObject(
            job.as_raw_handle(),
            JOB_OBJECT_EXTENDED_LIMIT_INFORMATION,
            (&raw const information).cast(),
            size_of::<ExtendedLimitInformation>() as u32,
        ) != 0
    }
}

/// Kills `job` with all its processes, and `child` should it be outside of
/// it.
fn kill_tree(child: &mut Child, job: Option<&OwnedHandle>) {
    if let Some(job) = job {
        terminate(job);
    }

    let _ = child.kill();
}

/// Kills all processes of `job`.
pub(super) fn terminate(job: &OwnedHandle) {
    // SAFETY: `job` is an open job handle.
    unsafe { TerminateJobObject(job.as_raw_handle(), 1) };
}

fn communicate(
    child: &mut Child,
    job: Option<&OwnedHandle>,
    mut channels: [Channel; 3],
    mut input: &[u8],
    deadline: Option<Instant>,
    outputs: &mut Outputs,
) -> std::io::Result<Option<ExitStatus>> {
    let mut grace_end = None;
    let mut failure = None;
    let mut first = 0;

    let kill = loop {
        if input.is_empty() {
            channels[0].pipe = None;
        }

        if channels[0].is_idle() {
            failure = failure.or(channels[0].start_write(input).err());
        }

        for channel in &mut channels[1..] {
            if channel.is_idle() {
                failure = failure.or(channel.start_read().err());
            }
        }

        let mut handles = Vec::with_capacity(4);
        let mut sources = Vec::with_capacity(4);

        // WHY: a wait reports the first signalled handle, so they take turns
        // to be first, so that a busy pipe does not starve the others.
        for index in (0..channels.len()).map(|offset| (first + offset) % channels.len()) {
            if channels[index].pending {
                handles.push(channels[index].event.as_raw_handle());
                sources.push(index);
            }
        }

        first = (first + 1) % channels.len();

        if grace_end.is_none() {
            handles.push(child.as_raw_handle());
            sources.push(PROCESS);
        }

        if handles.is_empty() {
            break false;
        }

        let timeout = match grace_end.or(deadline).map(millis_until) {
            Some(0) => break grace_end.is_none(),
            Some(millis) => millis,
            None => INFINITE,
        };

        // WHY: a timeout matches no source, so the loop checks the deadline
        // again.
        // SAFETY: `handles` are open handles alive for the duration of the
        // call.
        let signalled =
            unsafe { WaitForMultipleObjects(handles.len() as u32, handles.as_ptr(), 0, timeout) };

        let wait_failed = record_wait_failure(signalled, &mut failure);

        let finished = match sources.get(signalled as usize).copied() {
            Some(PROCESS) => {
                grace_end = Some(Instant::now() + EXIT_GRACE);
                Ok(())
            }
            Some(0) => channels[0].finish().map(|written| {
                input = written.map_or(&[][..], |written| &input[written..]);
            }),
            Some(1) => channels[1].finish().map(|read| {
                if let Some(read) = read {
                    outputs.push(OutputStream::Stdout, &channels[1].buffer[..read]);
                }
            }),
            Some(_) => channels[2].finish().map(|read| {
                if let Some(read) = read {
                    outputs.push(OutputStream::Stderr, &channels[2].buffer[..read]);
                }
            }),
            None => Ok(()),
        };
        failure = failure.or(finished.err());

        if wait_failed || outputs.failed() {
            break !wait_failed && outputs.failed();
        }
    };

    // WHY: closed before waiting, so that a child writing to a pipe no
    // longer read is not blocked.
    drop(channels);

    let status = if kill {
        kill_tree(child, job);
        child.wait().map(|_| None)
    } else {
        if let Some(job) = job {
            release(job);
        }
        child.wait().map(Some)
    };

    status.and_then(|status| failure.map_or(Ok(status), Err))
}

fn wait_failed(signalled: u32) -> Option<std::io::Error> {
    (signalled == WAIT_FAILED).then(std::io::Error::last_os_error)
}

fn record_wait_failure(signalled: u32, failure: &mut Option<std::io::Error>) -> bool {
    if let Some(error) = wait_failed(signalled) {
        *failure = failure.take().or(Some(error));
        true
    } else {
        false
    }
}

/// Returns `None`, a process on Windows is terminated with an exit code.
pub(super) fn exit_signal(_status: &ExitStatus) -> Option<i32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    mod wait_failed {
        use super::*;

        #[test]
        fn wait_failed_should_return_error_only_for_wait_failed() {
            let _ = std::fs::File::open("Z:\\agplatform_missing_wait_failed");

            assert!(wait_failed(WAIT_FAILED).is_some());
            assert!(wait_failed(0).is_none());
        }

        #[test]
        fn wait_failed_record_wait_failure_should_record_only_wait_failed() {
            let _ = std::fs::File::open("Z:\\agplatform_missing_record_wait_failed");
            let mut failure = None;

            assert!(record_wait_failure(WAIT_FAILED, &mut failure));
            assert!(failure.is_some());
            assert!(!record_wait_failure(0, &mut failure));
        }
    }

    mod channel {
        use super::*;

        mod started {
            use super::*;

            #[test]
            fn started_without_io_pending_should_fail_and_close_pipe() {
                let _ = std::fs::File::open("Z:\\agplatform_missing_started");
                let mut channel = Channel::new(None).unwrap();

                let error = channel.started(0).unwrap_err();

                assert!(channel.pipe.is_none());
                assert_ne!(error.raw_os_error(), Some(ERROR_IO_PENDING));
            }
        }
    }

    mod kill_tree {
        use super::*;
        use std::time::Duration;
        use std::time::Instant;

        #[test]
        fn kill_tree_with_job_should_terminate_child() {
            // GIVEN a running child and an explicit job handle
            let mut child = Command::new("cmd")
                .args(["/C", "ping -n 11 127.0.0.1 >NUL"])
                .spawn()
                .unwrap();
            let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
            assert!(!handle.is_null());
            let job = unsafe { OwnedHandle::from_raw_handle(handle) };
            let start = Instant::now();

            // WHEN killing the tree with the job present
            kill_tree(&mut child, Some(&job));

            // THEN the child is terminated quickly
            assert!(child.wait().unwrap().code() != Some(0));
            assert!(start.elapsed() < Duration::from_secs(5));
        }

        #[test]
        fn kill_tree_without_job_should_terminate_child() {
            // GIVEN a running child without a job
            let mut child = Command::new("cmd")
                .args(["/C", "ping -n 11 127.0.0.1 >NUL"])
                .spawn()
                .unwrap();
            let start = Instant::now();

            // WHEN killing the tree without a job
            kill_tree(&mut child, None);

            // THEN the child is terminated quickly
            assert!(child.wait().unwrap().code() != Some(0));
            assert!(start.elapsed() < Duration::from_secs(5));
        }
    }

    mod job {
        use super::*;
        use std::time::Duration;
        use std::time::Instant;

        #[test]
        fn job_closed_should_kill_its_processes() {
            // GIVEN a running child in a job
            let mut child = Command::new("cmd")
                .args(["/C", "ping -n 11 127.0.0.1 >NUL"])
                .spawn()
                .unwrap();
            let job = job(child.as_raw_handle()).unwrap();
            let start = Instant::now();

            // WHEN the job is closed
            drop(job);

            // THEN the child is killed right away
            assert!(child.wait().unwrap().code().is_some());
            assert!(start.elapsed() < Duration::from_secs(5));
        }
    }

    mod release {
        use super::*;

        #[test]
        fn release_should_let_processes_outlive_job() {
            // GIVEN a running child in a released job
            let mut child = Command::new("cmd")
                .args(["/C", "ping -n 11 127.0.0.1 >NUL"])
                .spawn()
                .unwrap();
            let job = job(child.as_raw_handle()).unwrap();
            release(&job);

            // WHEN the job is closed
            drop(job);

            // THEN the child keeps running
            assert!(child.try_wait().unwrap().is_none());
            child.kill().unwrap();
            child.wait().unwrap();
        }
    }

    mod wait_with_output {
        use super::*;
        use std::process::Stdio;

        /// Returns the outputs passing the chunks into `streams`.
        fn outputs_into(streams: &mut (Vec<u8>, Vec<u8>)) -> Outputs<'_> {
            Outputs::new(|stream, data| {
                match stream {
                    OutputStream::Stdout => streams.0.extend_from_slice(data),
                    OutputStream::Stderr => streams.1.extend_from_slice(data),
                }
                Ok(())
            })
        }

        #[test]
        fn wait_with_output_child_should_pass_stdout_and_stderr_on() {
            // GIVEN a child writing to both output streams
            let mut child = Command::new("cmd")
                .args(["/C", "echo stdout & echo stderr 1>&2"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();

            // WHEN reading its output
            let mut streams = (Vec::new(), Vec::new());
            let mut outputs = outputs_into(&mut streams);
            let status = wait_with_output(&mut child, b"", None, &mut outputs).unwrap();
            outputs.finish().unwrap();

            // THEN the output is passed on from both streams
            assert!(status.unwrap().success());
            let (stdout, stderr) = streams;
            assert!(
                stdout
                    .windows(b"stdout".len())
                    .any(|chunk| chunk == b"stdout")
            );
            assert!(
                stderr
                    .windows(b"stderr".len())
                    .any(|chunk| chunk == b"stderr")
            );
        }

        #[test]
        fn wait_with_output_exited_child_should_close_broken_pipes() {
            // GIVEN a child that exited, breaking its pipes
            let mut child = Command::new("cmd")
                .args(["/C", "exit 0"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            child.wait().unwrap();

            // WHEN communicating with it
            let mut streams = (Vec::new(), Vec::new());
            let mut outputs = outputs_into(&mut streams);
            let status = wait_with_output(&mut child, b"input", None, &mut outputs).unwrap();
            outputs.finish().unwrap();

            // THEN the failing writes and reads close the pipes
            assert!(status.unwrap().success());
            assert_eq!(streams, (Vec::new(), Vec::new()));
        }
    }
}
