//! [internal]
//!
//! The Unix specific parts of [`ProcessImpl`](super::ProcessImpl).

use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::os::fd::AsRawFd;
#[cfg(target_os = "linux")]
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::ExitStatus;
#[cfg(target_os = "linux")]
use std::sync::atomic::AtomicU32;
#[cfg(target_os = "linux")]
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use super::EXIT_GRACE;
use super::OutputStream;
use super::Outputs;
use super::millis_until;

#[cfg(target_os = "linux")]
type NFds = std::ffi::c_ulong;
#[cfg(not(target_os = "linux"))]
type NFds = std::ffi::c_uint;

#[cfg(target_os = "linux")]
const PIPE_BUF: usize = 4096;
#[cfg(not(target_os = "linux"))]
const PIPE_BUF: usize = 512;

/// The default capacity of a Linux pipe, read and written at once.
const BUFFER_SIZE: usize = 64 * 1024;
const ENOENT: i32 = 2;
#[cfg(target_os = "linux")]
const ESRCH: i32 = 3;
const F_GETFL: i32 = 3;
const F_SETFL: i32 = 4;
#[cfg(target_os = "linux")]
const O_NONBLOCK: i32 = 0o4000;
#[cfg(not(target_os = "linux"))]
const O_NONBLOCK: i32 = 0x4;
const POLLIN: i16 = 0x1;
const POLLOUT: i16 = 0x4;
#[cfg(target_os = "linux")]
const PR_SET_PDEATHSIG: i32 = 1;
const SIGKILL: i32 = 9;
#[cfg(target_os = "linux")]
const SYS_PIDFD_OPEN: std::ffi::c_long = 434;
const WAIT_INTERVAL_MS: u32 = 10;

/// The current process as the parent of the children configured with
/// [`configure`], read by them in [`die_with_parent`].
#[cfg(target_os = "linux")]
static PARENT: AtomicU32 = AtomicU32::new(0);

#[repr(C)]
struct PollFd {
    fd: i32,
    events: i16,
    revents: i16,
}

unsafe extern "C" {
    fn fcntl(fd: i32, command: i32, ...) -> i32;
    #[cfg(target_os = "linux")]
    fn getppid() -> i32;
    fn kill(pid: i32, signal: i32) -> i32;
    fn poll(fds: *mut PollFd, count: NFds, timeout: i32) -> i32;
    #[cfg(target_os = "linux")]
    fn prctl(option: i32, ...) -> i32;
    #[cfg(target_os = "linux")]
    fn syscall(number: std::ffi::c_long, ...) -> std::ffi::c_long;
}

/// Makes the child the leader of a new process group when it may need to
/// be killed with its descendants, see [`kill_group`]. On Linux such a
/// child is killed when the thread starting it exits, as it no longer gets
/// the signals sent to the process group of the current process.
pub(super) fn configure(command: &mut Command, group: bool) {
    if group {
        std::os::unix::process::CommandExt::process_group(command, 0);

        #[cfg(target_os = "linux")]
        {
            PARENT.store(std::process::id(), Ordering::Relaxed);
            // SAFETY: `die_with_parent` makes only async-signal-safe calls.
            unsafe { std::os::unix::process::CommandExt::pre_exec(command, die_with_parent) };
        }
    }
}

/// Makes the calling child killed when the thread that started it exits,
/// failing if the [`PARENT`] that did is gone already.
#[cfg(target_os = "linux")]
fn die_with_parent() -> std::io::Result<()> {
    // WHY: setting a valid signal cannot fail.
    // SAFETY: PR_SET_PDEATHSIG takes the signal as its only argument.
    unsafe { prctl(PR_SET_PDEATHSIG, SIGKILL as std::ffi::c_ulong) };

    // WHY: a parent dying before the signal is set orphans the child
    // without killing it, the child then has a new parent.
    // SAFETY: getppid has no preconditions and cannot fail.
    (unsafe { getppid() } as u32 == PARENT.load(Ordering::Relaxed))
        .then_some(())
        .ok_or(std::io::Error::from_raw_os_error(ESRCH))
}

/// Returns a descriptor of `child` that becomes readable when it exits,
/// `None` when unsupported.
#[cfg(target_os = "linux")]
fn exit_fd(child: &Child) -> Option<OwnedFd> {
    // SAFETY: pidfd_open takes the pid and no flags and returns a new
    // descriptor, which is close-on-exec, or -1.
    let fd = unsafe { syscall(SYS_PIDFD_OPEN, child.id() as std::ffi::c_long, 0) };

    // SAFETY: `fd` is a new descriptor owned by nothing else.
    (fd >= 0).then(|| unsafe { OwnedFd::from_raw_fd(fd as i32) })
}

/// Returns `None`, the exit of `child` is polled instead.
#[cfg(not(target_os = "linux"))]
fn exit_fd(_child: &Child) -> Option<OwnedFd> {
    None
}

/// Returns the first executable `program` in the directories of `path`,
/// relative and empty (POSIX) ones resolved against `dir`.
///
/// # Errors
///
/// Returns the error of std not finding a program if there is none.
pub(super) fn find_program(program: &Path, path: &OsStr, dir: &Path) -> std::io::Result<PathBuf> {
    std::env::split_paths(path)
        .map(|entry| dir.join(entry).join(program))
        .find(|candidate| {
            std::fs::metadata(candidate).is_ok_and(|metadata| {
                metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
            })
        })
        .ok_or(std::io::Error::from_raw_os_error(ENOENT))
}

/// Writes `input` to the standard input of `child` and reads its standard
/// output and error into `outputs` on the current thread until they close,
/// then waits for it to exit. Once it exited they are read at most for
/// [`EXIT_GRACE`]. Past `deadline`, or once the sink of `outputs` fails, it
/// is killed with its descendants and `None` is returned as the status.
///
/// # Errors
///
/// Returns the first failure to use the pipes, once the child exited, as
/// its output would be incomplete. The pipes are closed on a failure, all
/// of them if they cannot be polled, rather than retried.
pub(super) fn wait_with_output(
    child: &mut Child,
    mut input: &[u8],
    deadline: Option<Instant>,
    outputs: &mut Outputs,
) -> std::io::Result<Option<ExitStatus>> {
    let mut stdin = child.stdin.take().map(OwnedFd::from).map(File::from);
    let mut stdout = child.stdout.take().map(OwnedFd::from).map(File::from);
    let mut stderr = child.stderr.take().map(OwnedFd::from).map(File::from);
    let mut exit = exit_fd(child);
    let mut grace_end = None;
    let mut buffer = vec![0; BUFFER_SIZE];
    let mut failure = None;
    // WHY: once poll reports a pipe writable, writing at most PIPE_BUF bytes
    // does not block, a non-blocking pipe takes as much as it can.
    let max_write = stdin
        .as_ref()
        .and_then(|stdin| set_nonblocking(stdin).ok())
        .map_or(PIPE_BUF, |()| BUFFER_SIZE);

    let kill = loop {
        if input.is_empty() {
            stdin = None;
        }

        if stdin.is_none() && stdout.is_none() && stderr.is_none() {
            break false;
        }

        let timeout = match grace_end.or(deadline).map(millis_until) {
            Some(0) => break grace_end.is_none(),
            Some(millis) => millis as i32,
            None => -1,
        };
        let mut fds = [
            poll_fd(&stdin, POLLOUT),
            poll_fd(&stdout, POLLIN),
            poll_fd(&stderr, POLLIN),
            poll_fd(&exit, POLLIN),
        ];

        // WHY: without an exit descriptor the exit is checked at least every
        // WAIT_INTERVAL_MS.
        let polled = poll_pipes(
            &mut fds,
            [&mut stdin, &mut stdout, &mut stderr],
            exit.as_ref().map_or(interval(timeout), |_| timeout),
        );
        failure = failure.or(polled.err());

        if grace_end.is_none()
            && (exit.is_none() || fds[3].revents != 0)
            && child.try_wait().ok().flatten().is_some()
        {
            grace_end = Some(Instant::now() + EXIT_GRACE);
            exit = None;
        }

        if fds[0].revents != 0 {
            failure = failure.or(write_chunk(&mut stdin, &mut input, max_write).err());
        }

        if fds[1].revents != 0 {
            failure =
                failure
                    .or(read_chunk(&mut stdout, &mut buffer, outputs, OutputStream::Stdout).err());
        }

        if fds[2].revents != 0 {
            failure =
                failure
                    .or(read_chunk(&mut stderr, &mut buffer, outputs, OutputStream::Stderr).err());
        }

        if outputs.failed() {
            break true;
        }
    };

    let status = if kill {
        kill_and_wait(child)
    } else {
        wait_timeout(child, deadline)
    };

    status.and_then(|status| failure.map_or(Ok(status), Err))
}

/// Returns the signal that terminated the process of `status`, if any.
pub(super) fn exit_signal(status: &ExitStatus) -> Option<i32> {
    std::os::unix::process::ExitStatusExt::signal(status)
}

/// Kills the process group led by the child `pid` configured with
/// [`configure`].
pub(super) fn kill_group(pid: u32) {
    // SAFETY: a negative pid signals the whole process group.
    let _ = unsafe { kill(-(pid as i32), SIGKILL) };
}

/// Returns the poll `timeout` in milliseconds limited to
/// [`WAIT_INTERVAL_MS`], which an infinite (negative) one is as well.
fn interval(timeout: i32) -> i32 {
    (timeout as u32).min(WAIT_INTERVAL_MS) as i32
}

/// Kills `child` with the process group it leads, if any.
fn kill_and_wait(child: &mut Child) -> std::io::Result<Option<ExitStatus>> {
    kill_group(child.id());
    let _ = child.kill();
    child.wait().map(|_| None)
}

fn poll_fd<T: AsRawFd>(pipe: &Option<T>, events: i16) -> PollFd {
    PollFd {
        fd: pipe.as_ref().map_or(-1, AsRawFd::as_raw_fd),
        events,
        revents: 0,
    }
}

/// Polls the descriptors `fds` of the `pipes` (and the exit of a child)
/// for at most `timeout` milliseconds, negative for no limit, closing the
/// `pipes` if they cannot be polled. An interrupted poll is left to be
/// retried.
fn poll_pipes(
    fds: &mut [PollFd],
    pipes: [&mut Option<File>; 3],
    timeout: i32,
) -> std::io::Result<()> {
    // SAFETY: `fds` is a slice of pollfd alive for the duration of the call.
    // Closed pipes have a negative fd, which poll ignores.
    let result = unsafe { poll(fds.as_mut_ptr(), fds.len() as NFds, timeout) };
    let error = std::io::Error::last_os_error();

    (result >= 0 || is_transient(&error))
        .then_some(())
        .ok_or(error)
        .inspect_err(|_| pipes.into_iter().for_each(|pipe| *pipe = None))
}

/// Returns `true` for an `error` of an operation that can be retried.
fn is_transient(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock
    )
}

/// Reads what is available from `pipe` into `outputs` as its `stream`,
/// closing it at its end or when it fails.
fn read_chunk(
    pipe: &mut Option<File>,
    buffer: &mut [u8],
    outputs: &mut Outputs,
    stream: OutputStream,
) -> std::io::Result<()> {
    match pipe.as_mut().map(|pipe| pipe.read(buffer)) {
        Some(Ok(read @ 1..)) => {
            outputs.push(stream, &buffer[..read]);
            Ok(())
        }
        Some(Err(error)) if is_transient(&error) => Ok(()),
        Some(Err(error)) => {
            *pipe = None;
            Err(error)
        }
        _ => {
            *pipe = None;
            Ok(())
        }
    }
}

/// Makes writing to `file` take what it can without blocking.
fn set_nonblocking(file: &File) -> std::io::Result<()> {
    let fd = file.as_raw_fd();
    // SAFETY: F_GETFL and F_SETFL get and set the status flags of the open
    // `fd`, returning -1 on failure.
    let set = unsafe {
        let flags = fcntl(fd, F_GETFL);
        flags >= 0 && fcntl(fd, F_SETFL, flags | O_NONBLOCK) >= 0
    };

    set.then_some(()).ok_or_else(std::io::Error::last_os_error)
}

/// Waits for `child` to exit at most until `deadline`, otherwise kills it
/// with its descendants and returns `None`.
fn wait_timeout(
    child: &mut Child,
    deadline: Option<Instant>,
) -> std::io::Result<Option<ExitStatus>> {
    let Some(deadline) = deadline else {
        return child.wait().map(Some);
    };

    loop {
        match child.try_wait() {
            Ok(None) => match millis_until(deadline) {
                0 => return kill_and_wait(child),
                millis => {
                    std::thread::sleep(Duration::from_millis(millis.min(WAIT_INTERVAL_MS).into()))
                }
            },
            result => return result,
        }
    }
}

/// Writes the next chunk of `input` to `stdin`, at most `max` bytes, and
/// leaves the rest in `input`, none once `stdin` is closed by the child or
/// fails.
fn write_chunk(stdin: &mut Option<File>, input: &mut &[u8], max: usize) -> std::io::Result<()> {
    match stdin
        .as_mut()
        .map(|stdin| stdin.write(&input[..input.len().min(max)]))
    {
        Some(Ok(written)) => {
            *input = &input[written..];
            Ok(())
        }
        Some(Err(error)) if is_transient(&error) => Ok(()),
        Some(Err(error)) if error.kind() != std::io::ErrorKind::BrokenPipe => {
            *input = &[];
            Err(error)
        }
        _ => {
            *input = &[];
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestDir;

    #[cfg(target_os = "linux")]
    mod die_with_parent {
        use super::*;

        #[test]
        fn die_with_parent_orphaned_should_fail() {
            // GIVEN the current process as the parent, which is not its own
            PARENT.store(std::process::id(), Ordering::Relaxed);

            // WHEN setting the parent death signal of the current process
            let result = die_with_parent();
            // WHY: the signal set by `die_with_parent` is reset for the test thread.
            unsafe { prctl(PR_SET_PDEATHSIG, 0 as std::ffi::c_ulong) };

            // THEN the parent is found to be gone
            assert_eq!(result.unwrap_err().raw_os_error(), Some(ESRCH));
        }
    }

    mod find_program {
        use super::*;

        fn write(path: &Path, mode: u32) {
            std::fs::write(path, "").unwrap();
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
        }

        #[test]
        fn find_program_should_return_first_executable_match() {
            // GIVEN a not executable file, a directory and two executables
            // of the name in four directories
            let dir = TestDir::new();
            for name in ["file", "dir", "first", "second"] {
                std::fs::create_dir(dir.path().join(name)).unwrap();
            }
            write(&dir.path().join("file/tool"), 0o644);
            std::fs::create_dir(dir.path().join("dir/tool")).unwrap();
            write(&dir.path().join("first/tool"), 0o755);
            write(&dir.path().join("second/tool"), 0o755);

            // WHEN finding it in all of them, some relative to the directory
            let found = find_program(
                Path::new("tool"),
                OsStr::new(&format!("file:{}/dir:first:second", dir.path().display())),
                dir.path(),
            );

            // THEN the first executable is found
            assert_eq!(found.unwrap(), dir.path().join("first/tool"));
        }

        #[test]
        fn find_program_empty_entry_should_search_dir() {
            let dir = TestDir::new();
            write(&dir.path().join("tool"), 0o700);

            let found = find_program(
                Path::new("tool"),
                OsStr::new("/agplatform_missing:"),
                dir.path(),
            );

            assert_eq!(found.unwrap(), dir.path().join("tool"));
        }

        #[test]
        fn find_program_missing_should_return_not_found() {
            let dir = TestDir::new();

            let error = find_program(Path::new("tool"), OsStr::new("bin"), dir.path()).unwrap_err();

            assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
            assert_eq!(error.raw_os_error(), Some(ENOENT));
        }
    }

    mod interval {
        use super::*;

        #[test]
        fn interval_should_limit_timeout_to_wait_interval() {
            for (timeout, expected) in [(-1, 10), (0, 0), (5, 5), (10_000, 10)] {
                assert_eq!(interval(timeout), expected, "timeout: {timeout}");
            }
        }
    }

    /// Calls `f` with the outputs of a run, returning its result with the
    /// standard output passed on.
    fn with_outputs<T>(f: impl FnOnce(&mut Outputs) -> T) -> (T, Vec<u8>) {
        let mut stdout = Vec::new();
        let result = f(&mut Outputs::new(|_, data| {
            stdout.extend_from_slice(data);
            Ok(())
        }));
        (result, stdout)
    }

    /// Returns the ends of a new pipe, the writing one non-blocking.
    fn nonblocking_pipe() -> (File, File) {
        let (reader, writer) = std::io::pipe().unwrap();
        let writer = File::from(OwnedFd::from(writer));
        set_nonblocking(&writer).unwrap();
        (File::from(OwnedFd::from(reader)), writer)
    }

    mod poll_pipes {
        use super::*;

        #[test]
        fn poll_pipes_failing_should_close_pipes() {
            // GIVEN more descriptors than a process may have open
            let mut fds: Vec<PollFd> = (0..1 << 21)
                .map(|_| PollFd {
                    fd: -1,
                    events: 0,
                    revents: 0,
                })
                .collect();
            let mut pipes = [
                Some(File::open("Cargo.toml").unwrap()),
                Some(File::open("Cargo.toml").unwrap()),
                None,
            ];
            let [stdin, stdout, stderr] = &mut pipes;

            // WHEN polling them
            let result = poll_pipes(&mut fds, [stdin, stdout, stderr], 0);

            // THEN it fails and the pipes are closed
            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::InvalidInput);
            assert!(pipes.iter().all(Option::is_none));
        }
    }

    mod read_chunk {
        use super::*;

        #[test]
        fn read_chunk_should_pass_data_read_on() {
            let (reader, mut writer) = nonblocking_pipe();
            writer.write_all(b"out").unwrap();
            let mut pipe = Some(reader);

            let ((), stdout) = with_outputs(|outputs| {
                read_chunk(&mut pipe, &mut [0; 8], outputs, OutputStream::Stdout).unwrap();
            });

            assert_eq!(stdout, b"out");
            assert!(pipe.is_some());
        }

        #[test]
        fn read_chunk_with_nothing_to_read_yet_should_keep_pipe() {
            let (reader, _writer) = nonblocking_pipe();
            set_nonblocking(&reader).unwrap();
            let mut pipe = Some(reader);

            let (result, _) = with_outputs(|outputs| {
                read_chunk(&mut pipe, &mut [0; 8], outputs, OutputStream::Stdout)
            });

            assert!(result.is_ok());
            assert!(pipe.is_some());
        }

        #[test]
        fn read_chunk_failing_should_close_pipe_and_fail() {
            let mut pipe = Some(File::open(".").unwrap());

            let (result, _) = with_outputs(|outputs| {
                read_chunk(&mut pipe, &mut [0; 8], outputs, OutputStream::Stdout)
            });

            assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::IsADirectory);
            assert!(pipe.is_none());
        }
    }

    mod write_chunk {
        use super::*;

        #[test]
        fn write_chunk_to_full_pipe_should_keep_input() {
            // GIVEN a full pipe
            let (_reader, mut writer) = nonblocking_pipe();
            while writer.write(&[0; PIPE_BUF]).is_ok() {}
            let mut stdin = Some(writer);
            let mut input: &[u8] = b"input";

            // WHEN writing more to it
            let result = write_chunk(&mut stdin, &mut input, BUFFER_SIZE);

            // THEN the input is left to be written
            assert!(result.is_ok());
            assert_eq!(input, b"input");
        }

        #[test]
        fn write_chunk_to_closed_pipe_should_drop_input() {
            let (reader, writer) = nonblocking_pipe();
            drop(reader);
            let mut input: &[u8] = b"input";

            let result = write_chunk(&mut Some(writer), &mut input, BUFFER_SIZE);

            assert!(result.is_ok());
            assert!(input.is_empty());
        }

        #[test]
        fn write_chunk_failing_should_drop_input_and_fail() {
            let mut input: &[u8] = b"input";

            let result = write_chunk(
                &mut Some(File::open("Cargo.toml").unwrap()),
                &mut input,
                BUFFER_SIZE,
            );

            assert!(result.is_err());
            assert!(input.is_empty());
        }
    }

    mod wait_timeout {
        use super::*;

        #[test]
        fn wait_timeout_exceeding_deadline_should_kill_child() {
            // GIVEN a running child leading its process group
            let mut command = Command::new("sleep");
            command.arg("10");
            configure(&mut command, true);
            let mut child = command.spawn().unwrap();
            let start = Instant::now();

            // WHEN its deadline passes
            let status = wait_timeout(&mut child, Some(start + Duration::from_millis(100)));

            // THEN it is killed right away
            assert_eq!(status.unwrap(), None);
            assert!(start.elapsed() < Duration::from_secs(5));
        }
    }
}
