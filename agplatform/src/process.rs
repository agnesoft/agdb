use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use crate::Result;
use crate::env::EnvImpl;
#[cfg(feature = "tokio")]
use crate::interceptor::AsyncExecutor;
use crate::interceptor::BlockingExecutor;
use crate::interceptor::Interceptor;
use crate::interceptor::Interceptors;
use crate::interceptor::Run;
use crate::interceptor::Sink;
use crate::utils::Runs;
use crate::utils::conclude;

macro_rules! process_error {
    ($($arg:tt)*) => {
        $crate::Error::process(format!($($arg)*))
    };
}

mod cache;
mod command_output;
mod process_command;
mod process_impl;
mod std_stream;
#[cfg(feature = "tokio")]
mod tokio_process;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

pub use cache::CommandCache;
pub use command_output::CommandOutput;
pub use process_command::CommandOptions;
pub use process_command::ProcessCommand;
pub use process_impl::ProcessImpl;
pub use std_stream::StdStream;
#[cfg(unix)]
use unix::{configure, exit_signal, wait_with_output};
#[cfg(windows)]
use windows::{configure, exit_signal, wait_with_output};

/// [internal]
///
/// Receives the output of a child process as it arrives, see
/// [`Process::run_blocking_with_sink`].
pub(crate) type OutputSink<'a> = dyn FnMut(OutputStream, &[u8]) -> Result + Send + 'a;

/// The output of a child process a chunk is from, see [`Process::run_blocking_with_sink`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputStream {
    /// The standard output.
    Stdout,
    /// The standard error.
    Stderr,
}

/// [internal]
///
/// The output of an attempt as it arrives, passed on with `emit`, e.g.
/// through the interceptors to the sink. The first error of `emit` ends
/// the attempt.
pub(crate) struct Outputs<'a> {
    emit: Box<OutputSink<'a>>,
    error: Option<crate::Error>,
}

/// Abstract process interface.
///
/// Runs child processes described by [`std::process::Command`] to
/// completion. Child processes run in the configured current directory
/// unless the command sets an absolute one, a relative one is resolved
/// against it. They get the configured environment variables, updated by
/// the variables the command sets or removes. Each run passes through
/// [`Interceptor`]s, see [`ProcessCommand`].
///
/// Usually used through [`CommandExt`] rather than directly.
///
/// The two provided implementations are `ProcessImpl` (delegates to
/// [`std::process`]) and
/// [`TestProcess`](crate::test_platform::test_process::TestProcess)
/// (purely in-memory, behind the `testing` feature).
///
/// Processes are [`Sync`] so that the futures of
/// [`run_async`](Self::run_async) borrowing them are [`Send`].
pub trait Process: Send + Sync {
    /// Runs `run`, a command with its [`CommandOptions`], to completion and
    /// returns its output. A plain [`Command`] runs with the default
    /// options, a [`Run`] (e.g. from [`CommandExt`]) with its own.
    ///
    /// Only the program, the arguments, the environment variables and the
    /// current directory are taken from `command`, which is left
    /// unchanged. Other settings such as
    /// [`env_clear`](Command::env_clear) (use [`CommandOptions::clear_env`]
    /// instead) or the platform specific ones are not observable and
    /// therefore ignored. The standard input of the
    /// child process is null (or [`CommandOptions::input`]), its standard
    /// output and error are captured (and streamed with
    /// [`CommandOptions::stream`]), unless it is [`CommandOptions::interactive`].
    ///
    /// A program given by a relative path with a separator, e.g.
    /// `./tool.sh`, is resolved against the directory the command runs in.
    /// On Unix a bare program name is looked up in the `PATH` the child
    /// process would get without [`CommandOptions::clear_env`], unless set
    /// by the command, so changing it or clearing the environment does not
    /// prevent finding programs.
    ///
    /// The run ends when the child process exits. Its output is read until
    /// it closes, or at most 100 ms after the exit if descendants,
    /// e.g. daemons, hold it open. They keep running and the rest of their
    /// output is dropped.
    ///
    /// # Errors
    ///
    /// The errors name only the program of `command`, as its arguments may
    /// hold secrets. An [`Interceptor`] gets the whole command, e.g. to log
    /// it redacted.
    ///
    /// Returns an error if:
    /// - [`CommandOptions::interactive`] is combined with
    ///   [`CommandOptions::input`] or [`CommandOptions::timeout`].
    /// - The process cannot be started.
    /// - The output is not a [success](CommandOutput::success), e.g. the
    ///   process exits with a non-zero code, is terminated without an exit
    ///   code or times out. The [`CommandOutput`] is returned by
    ///   [`Error::command_output`](crate::Error::command_output).
    /// - An interceptor returns an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandExt, CommandOptions, CommandOutput, Platform, Process, ProcessCommand};
    /// use std::process::Command;
    /// use std::time::Duration;
    ///
    /// let platform = agplatform::platform();
    /// let process = platform.process();
    /// # let process = agplatform::TestProcess::new();
    /// # process.set_output("git", CommandOutput { stdout: b"git version 2.0".to_vec(), ..Default::default() });
    ///
    /// let output = process.run_blocking(Command::new("git").arg("--version")).unwrap();
    /// assert!(output.stdout_str().unwrap().starts_with("git version"));
    ///
    /// let mut fetch = Command::new("git");
    /// fetch.arg("fetch");
    /// process.run_blocking(fetch.timeout(Duration::from_secs(60))).unwrap();
    ///
    /// let options = CommandOptions { silent: true, ..Default::default() };
    /// process.run_blocking(ProcessCommand::with_options(&fetch, options)).unwrap();
    /// ```
    fn run_blocking(&self, run: impl Into<Run<ProcessCommand>>) -> Result<CommandOutput>;

    /// Runs `run` like [`Self::run_blocking`], passing the standard output and error
    /// to `sink` as they arrive, each chunk with the [`OutputStream`] it is
    /// from, instead of capturing them, e.g. for output too large to hold
    /// in memory. The [`CommandOutput`] then has no output, nor does the
    /// error of a failed run show any. The chunks pass through the
    /// interceptors before, see [`Interceptor::chunk`].
    ///
    /// A closure `|stream: OutputStream, chunk: &[u8]|` is a [`Sink`]. An
    /// async sink awaiting, e.g. a full channel, holds the reading of the
    /// output back. Its futures are polled on the current thread, so they
    /// must not need a runtime, see
    /// [`BlockingExecutor`](crate::interceptor::BlockingExecutor).
    ///
    /// The `sink` gets the output whether the run succeeds or not, which is
    /// known only once it ends. The chunks are as read, not split into
    /// lines. An attempt failing after a chunk was passed to `sink` is not
    /// retried by [`Retry`](crate::interceptor::Retry), as `sink` would get
    /// the output again from its start, see [`Run::streamed`].
    ///
    /// # Errors
    ///
    /// As [`Self::run_blocking`], or the error returned by `sink`, which ends the
    /// attempt, killing the child process like [`CommandOptions::timeout`].
    /// On Unix its descendants are killed with it only when it leads a
    /// process group, i.e. with a [`CommandOptions::timeout`], even
    /// [`Duration::MAX`](std::time::Duration::MAX), or when run by
    /// [`Self::run_async_with_sink`]. Fails as well if the run is [`CommandOptions::interactive`] or
    /// [`CommandOptions::silent`], which leave no output to pass.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandOutput, OutputStream, Platform, Process};
    /// use std::process::Command;
    ///
    /// let platform = agplatform::platform();
    /// let process = platform.process();
    /// # let process = agplatform::TestProcess::new();
    /// # process.set_output("git", CommandOutput { stdout: b"archive".to_vec(), ..Default::default() });
    /// let mut archive = Vec::new();
    ///
    /// let output = process
    ///     .run_blocking_with_sink(Command::new("git").args(["archive", "HEAD"]), &mut |stream: OutputStream, chunk: &[u8]| {
    ///         if stream == OutputStream::Stdout {
    ///             archive.extend_from_slice(chunk);
    ///         }
    ///         Ok(())
    ///     })
    ///     .unwrap();
    /// assert!(output.stdout.is_empty());
    /// assert_eq!(archive, b"archive");
    /// ```
    fn run_blocking_with_sink(
        &self,
        run: impl Into<Run<ProcessCommand>>,
        sink: &mut dyn Sink<ProcessCommand>,
    ) -> Result<CommandOutput>;

    /// Enabled by the `tokio` feature flag.
    ///
    /// Runs `run` to completion like [`Self::run_blocking`], without blocking the
    /// current thread. Must be awaited
    /// on a [`tokio`] runtime with I/O and time enabled.
    ///
    /// Dropping the future before it completes, e.g. to cancel the run,
    /// kills the child process with its process group on Unix, or its job
    /// on Windows, like [`CommandOptions::timeout`]. Therefore on Unix the
    /// child process always leads a new process group, unless it is
    /// [`CommandOptions::interactive`], when only the child process is
    /// killed. The interceptors are told by [`Interceptor::cancelled`].
    ///
    /// A child process leading a process group is also killed when the
    /// current process dies, see [`CommandOptions::timeout`]. It is started
    /// on the thread polling the future, a runtime worker or the one
    /// blocking on it, which outlives the run unless the runtime shuts down,
    /// cancelling the run anyway. The exception on Linux is a worker that
    /// calls `tokio::task::block_in_place`: it is replaced and continues as
    /// a blocking thread, which exits once idle (after 10 s by default),
    /// killing the children started on it that still run.
    ///
    /// # Errors
    ///
    /// As [`Self::run_blocking`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandOutput, Platform, Process};
    /// use std::process::Command;
    ///
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// let platform = agplatform::platform();
    /// let process = platform.process();
    /// # let process = agplatform::TestProcess::new();
    /// # process.set_output("git", CommandOutput { stdout: b"git version 2.0".to_vec(), ..Default::default() });
    ///
    /// let output = process.run_async(Command::new("git").arg("--version")).await.unwrap();
    /// assert!(output.stdout_str().unwrap().starts_with("git version"));
    /// # });
    /// ```
    #[cfg(feature = "tokio")]
    fn run_async(
        &self,
        run: impl Into<Run<ProcessCommand>>,
    ) -> impl Future<Output = Result<CommandOutput>> + Send;

    /// Enabled by the `tokio` feature flag.
    ///
    /// Runs `run` like [`Self::run_async`], passing its output to `sink`
    /// like [`Self::run_blocking_with_sink`], its futures awaited on the
    /// runtime of the run.
    ///
    /// # Errors
    ///
    /// As [`Self::run_blocking_with_sink`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandOutput, OutputStream, Platform, Process};
    /// use std::process::Command;
    ///
    /// # tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap().block_on(async {
    /// let platform = agplatform::platform();
    /// let process = platform.process();
    /// # let process = agplatform::TestProcess::new();
    /// # process.set_output("cargo", CommandOutput { stderr: b"Compiling".to_vec(), ..Default::default() });
    /// let mut log = Vec::new();
    ///
    /// process
    ///     .run_async_with_sink(Command::new("cargo").arg("build"), &mut |_: OutputStream, chunk: &[u8]| {
    ///         log.extend_from_slice(chunk);
    ///         Ok(())
    ///     })
    ///     .await
    ///     .unwrap();
    /// assert_eq!(log, b"Compiling");
    /// # });
    /// ```
    #[cfg(feature = "tokio")]
    fn run_async_with_sink(
        &self,
        run: impl Into<Run<ProcessCommand>>,
        sink: &mut dyn Sink<ProcessCommand>,
    ) -> impl Future<Output = Result<CommandOutput>> + Send;

    /// Returns the interceptors of the runs, e.g. to set them with changes
    /// on a run, see [`Run::interceptors`](Run#method.interceptors).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::interceptor::Retry;
    /// use agplatform::{CommandExt, CommandOutput, Process, TestProcess};
    /// use std::process::Command;
    /// use std::time::Duration;
    ///
    /// let process = TestProcess::new();
    /// let failure = CommandOutput { status: Some(1), ..Default::default() };
    /// process.set_outputs("git", [failure, CommandOutput::default()]);
    ///
    /// let mut interceptors = process.interceptors();
    /// interceptors.insert(0, Retry::new(3, Duration::from_secs(1)));
    /// Command::new("git").arg("fetch").interceptors(interceptors).run_blocking(&process).unwrap();
    ///
    /// assert_eq!(process.calls(), ["git fetch", "git fetch"]);
    /// ```
    fn interceptors(&self) -> Interceptors<ProcessCommand>;

    /// Removes the interceptors, [`StdStream`] by default, for the runs
    /// starting afterwards, e.g. before adding others with
    /// [`Self::add_interceptor`]. Without [`StdStream`]
    /// [`CommandOptions::stream`] has no effect.
    fn clear_interceptors(&self);

    /// Adds `interceptor` as the innermost one for the runs starting
    /// afterwards. Its state to be read later, e.g. what it recorded, is
    /// shared through an [`Arc`](std::sync::Arc) it holds.
    ///
    /// See [`ProcessCommand`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::interceptor::{DryRun, Retry};
    /// use agplatform::{CommandExt, CommandOutput, DryRunOutput, Process, TestProcess};
    /// use std::process::Command;
    /// use std::time::Duration;
    ///
    /// let process = TestProcess::new();
    /// process.add_interceptor(Retry::new(3, Duration::from_secs(5)).when(|_, result| {
    ///     result
    ///         .as_ref()
    ///         .is_ok_and(|output: &CommandOutput| output.stderr_lossy().contains("Could not resolve host"))
    /// }));
    /// process.add_interceptor(DryRun);
    ///
    /// let applied = Command::new("kubectl").arg("apply").run_blocking(&process).unwrap();
    /// let pods = Command::new("kubectl")
    ///     .args(["get", "pods"])
    ///     .with(DryRunOutput(CommandOutput { stdout: b"pod-1".to_vec(), ..Default::default() }))
    ///     .run_blocking(&process)
    ///     .unwrap();
    ///
    /// assert!(applied.stdout.is_empty());
    /// assert_eq!(pods.stdout, b"pod-1");
    /// assert!(process.calls().is_empty());
    /// ```
    fn add_interceptor(&self, interceptor: impl Interceptor<ProcessCommand>);
}

/// Extends [`std::process::Command`] with the ability to run it through a
/// [`Process`].
///
/// # Examples
///
/// ```rust
/// use agplatform::{CommandExt, CommandOutput, Platform};
/// use std::process::Command;
/// use std::time::Duration;
///
/// let platform = agplatform::platform();
/// let process = platform.process();
/// # let process = &agplatform::TestProcess::new();
/// # process.set_output("", CommandOutput::default());
///
/// Command::new("git").arg("status").run_blocking(process).unwrap();
/// Command::new("cargo")
///     .arg("build")
///     .stream()
///     .timeout(Duration::from_secs(600))
///     .run_blocking(process)
///     .unwrap();
/// ```
pub trait CommandExt {
    /// Runs the command with the default [`CommandOptions`].
    ///
    /// See [`Process::run_blocking`].
    fn run_blocking<P: Process + ?Sized>(&self, process: &P) -> Result<CommandOutput>;

    /// Runs the command with the default [`CommandOptions`], passing its
    /// output to `sink`.
    ///
    /// See [`Process::run_blocking_with_sink`].
    fn run_blocking_with_sink<P: Process + ?Sized>(
        &self,
        process: &P,
        sink: &mut dyn Sink<ProcessCommand>,
    ) -> Result<CommandOutput>;

    /// Enabled by the `tokio` feature flag.
    ///
    /// Runs the command asynchronously with the default [`CommandOptions`].
    ///
    /// See [`Process::run_async`].
    #[cfg(feature = "tokio")]
    fn run_async<P: Process + ?Sized>(
        &self,
        process: &P,
    ) -> impl Future<Output = Result<CommandOutput>> + Send;

    /// Enabled by the `tokio` feature flag.
    ///
    /// Runs the command asynchronously with the default [`CommandOptions`],
    /// passing its output to `sink`.
    ///
    /// See [`Process::run_async_with_sink`].
    #[cfg(feature = "tokio")]
    fn run_async_with_sink<P: Process + ?Sized>(
        &self,
        process: &P,
        sink: &mut dyn Sink<ProcessCommand>,
    ) -> impl Future<Output = Result<CommandOutput>> + Send;

    /// Returns a run of the command without the configured environment
    /// variables.
    ///
    /// See [`CommandOptions::clear_env`].
    fn clear_env(&self) -> Run<ProcessCommand>;

    /// Returns a run of the command whose output shows its last `lines`
    /// when displayed.
    ///
    /// See [`CommandOptions::display_lines`].
    fn display_lines(&self, lines: usize) -> Run<ProcessCommand>;

    /// Returns a run of the command whose output is discarded.
    ///
    /// See [`CommandOptions::silent`].
    fn silent(&self) -> Run<ProcessCommand>;

    /// Returns a run of the command reading `input`.
    ///
    /// See [`CommandOptions::input`].
    fn input<T: Into<Vec<u8>>>(&self, input: T) -> Run<ProcessCommand>;

    /// Returns a run of the command connected to the terminal.
    ///
    /// See [`CommandOptions::interactive`].
    fn interactive(&self) -> Run<ProcessCommand>;

    /// Returns a run of the command that streams its output.
    ///
    /// See [`CommandOptions::stream`].
    fn stream(&self) -> Run<ProcessCommand>;

    /// Returns a run of the command limited to `timeout`.
    ///
    /// See [`CommandOptions::timeout`].
    fn timeout(&self, timeout: Duration) -> Run<ProcessCommand>;

    /// Returns a run of the command with its own `interceptors`.
    ///
    /// See [`Run::interceptors`](Run#method.interceptors).
    fn interceptors(&self, interceptors: Interceptors<ProcessCommand>) -> Run<ProcessCommand>;

    /// Returns a run of the command with `value` for the interceptors.
    ///
    /// See [`Run::with`].
    fn with<T: std::any::Any + Send + Sync>(&self, value: T) -> Run<ProcessCommand>;
}

impl CommandExt for Command {
    fn run_blocking<P: Process + ?Sized>(&self, process: &P) -> Result<CommandOutput> {
        process.run_blocking(self)
    }

    fn run_blocking_with_sink<P: Process + ?Sized>(
        &self,
        process: &P,
        sink: &mut dyn Sink<ProcessCommand>,
    ) -> Result<CommandOutput> {
        process.run_blocking_with_sink(self, sink)
    }

    #[cfg(feature = "tokio")]
    fn run_async<P: Process + ?Sized>(
        &self,
        process: &P,
    ) -> impl Future<Output = Result<CommandOutput>> + Send {
        process.run_async(self)
    }

    #[cfg(feature = "tokio")]
    fn run_async_with_sink<P: Process + ?Sized>(
        &self,
        process: &P,
        sink: &mut dyn Sink<ProcessCommand>,
    ) -> impl Future<Output = Result<CommandOutput>> + Send {
        process.run_async_with_sink(self, sink)
    }

    fn clear_env(&self) -> Run<ProcessCommand> {
        Run::from(self).clear_env()
    }

    fn display_lines(&self, lines: usize) -> Run<ProcessCommand> {
        Run::from(self).display_lines(lines)
    }

    fn silent(&self) -> Run<ProcessCommand> {
        Run::from(self).silent()
    }

    fn input<T: Into<Vec<u8>>>(&self, input: T) -> Run<ProcessCommand> {
        Run::from(self).input(input)
    }

    fn interactive(&self) -> Run<ProcessCommand> {
        Run::from(self).interactive()
    }

    fn stream(&self) -> Run<ProcessCommand> {
        Run::from(self).stream()
    }

    fn timeout(&self, timeout: Duration) -> Run<ProcessCommand> {
        Run::from(self).timeout(timeout)
    }

    fn interceptors(&self, interceptors: Interceptors<ProcessCommand>) -> Run<ProcessCommand> {
        Run::from(self).interceptors(interceptors)
    }

    fn with<T: std::any::Any + Send + Sync>(&self, value: T) -> Run<ProcessCommand> {
        Run::from(self).with(value)
    }
}

/// [internal]
///
/// Returns the runs through [`StdStream`], the interceptors of a process by
/// default.
pub(crate) fn std_runs() -> Runs<ProcessCommand> {
    let mut interceptors = Interceptors::new();
    interceptors.push(StdStream);
    Runs::new(interceptors)
}

/// [internal]
///
/// Runs `run` by `executor` through `runs` in the current directory of
/// `env` with its variables, passing its output to `sink`, if any, and
/// returns its output, an error when it is not a success, see
/// [`Process::run_blocking`].
pub(crate) fn run_blocking<E: BlockingExecutor<Subject = ProcessCommand>>(
    runs: &Runs<ProcessCommand>,
    env: &EnvImpl,
    executor: &E,
    mut run: Run<ProcessCommand>,
    sink: Option<&mut dyn Sink<ProcessCommand>>,
) -> Result<CommandOutput> {
    run.subject_mut().resolve(env);
    let result = runs.run_blocking(executor, &mut run, sink);
    conclude(result, |output| failure(run.subject(), output))
}

/// [internal]
///
/// Runs `run` like [`run_blocking`] asynchronously, see
/// [`Process::run_async`].
#[cfg(feature = "tokio")]
pub(crate) async fn run_async<E: AsyncExecutor<Subject = ProcessCommand>>(
    runs: &Runs<ProcessCommand>,
    env: &EnvImpl,
    executor: &E,
    mut run: Run<ProcessCommand>,
    sink: Option<&mut dyn Sink<ProcessCommand>>,
) -> Result<CommandOutput> {
    run.subject_mut().resolve(env);
    let result = runs.run(executor, &mut run, sink).await;
    conclude(result, |output| failure(run.subject(), output))
}

/// [internal]
///
/// Returns the error of `command` failing with `output` as its source.
pub(crate) fn failure(command: &ProcessCommand, output: CommandOutput) -> crate::Error {
    let program = command.program();
    let error = match (output.timed_out, output.status) {
        (true, _) => process_error!("'{program}' timed out after {:?}", output.duration),
        (false, Some(0)) => process_error!("'{program}' was not accepted with exit code 0"),
        (false, Some(code)) => process_error!("'{program}' failed with exit code {code}"),
        (false, None) => match output.signal {
            Some(signal) => process_error!("'{program}' was terminated by signal {signal}"),
            None => process_error!("'{program}' was terminated without an exit code"),
        },
    };

    error.with_subject(program).with_source(output)
}

/// [internal]
///
/// Returns the error of `program` failing to be started or waited for with
/// the I/O `error` as its source.
pub(crate) fn io_error(program: &str, error: std::io::Error) -> crate::Error {
    process_error!("'{program}': {error}")
        .with_subject(program)
        .with_source(error)
}

const DISPLAY_LINES: usize = 10;

/// How long the output of a child process is read at most after it exited,
/// see [`Process::run_blocking`].
pub(crate) const EXIT_GRACE: Duration = Duration::from_millis(100);

impl<'a> Outputs<'a> {
    /// Creates the outputs passing the chunks on with `emit`.
    pub(crate) fn new(emit: impl FnMut(OutputStream, &[u8]) -> Result + Send + 'a) -> Self {
        Self {
            emit: Box::new(emit),
            error: None,
        }
    }

    /// Passes the `data` of the output `stream` of the child process on,
    /// unless passing a chunk on failed before.
    pub(crate) fn push(&mut self, stream: OutputStream, data: &[u8]) {
        if self.failed() {
            return;
        }

        if let Err(error) = (self.emit)(stream, data) {
            self.error = Some(error);
        }
    }

    /// Returns `true` once passing a chunk on failed, which ends the attempt.
    pub(crate) fn failed(&self) -> bool {
        self.error.is_some()
    }

    /// Returns the error of passing a chunk on if it failed.
    pub(crate) fn finish(self) -> Result {
        self.error.map_or(Ok(()), Err)
    }
}

/// [internal]
///
/// Returns the milliseconds until `deadline` rounded up, `0` when it has
/// passed, at most `i32::MAX`.
pub(crate) fn millis_until(deadline: Instant) -> u32 {
    deadline
        .saturating_duration_since(Instant::now())
        .as_nanos()
        .div_ceil(1_000_000)
        .min(i32::MAX as u128) as u32
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::Env;
    use crate::interceptor::BoxFuture;
    use crate::interceptor::Chunk;
    use crate::interceptor::Next;
    use crate::interceptor::Retry;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[cfg(unix)]
    pub(super) fn shell(script: &str) -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", script]);
        command
    }

    #[cfg(windows)]
    pub(super) fn shell(script: &str) -> Command {
        let mut command = Command::new("cmd");
        std::os::windows::process::CommandExt::raw_arg(command.arg("/C"), script);
        command
    }

    #[cfg(unix)]
    pub(super) const SHELL: &str = "sh";

    #[cfg(windows)]
    pub(super) const SHELL: &str = "cmd";

    #[cfg(unix)]
    pub(super) const SLEEP_THEN_ECHO: &str = "sleep 10; echo done";

    #[cfg(windows)]
    pub(super) const SLEEP_THEN_ECHO: &str = "ping -n 11 127.0.0.1 >NUL & echo done";

    /// Returns a sink collecting the standard output and error into
    /// `sunk`.
    pub(super) fn sink_into(
        sunk: &mut (Vec<u8>, Vec<u8>),
    ) -> impl FnMut(OutputStream, &[u8]) -> Result + Send + '_ {
        |stream, data| {
            match stream {
                OutputStream::Stdout => sunk.0.extend_from_slice(data),
                OutputStream::Stderr => sunk.1.extend_from_slice(data),
            }
            Ok(())
        }
    }

    #[cfg(unix)]
    pub(super) const INHERITED_VAR: &str = "HOME";

    #[cfg(windows)]
    pub(super) const INHERITED_VAR: &str = "USERPROFILE";

    #[cfg(unix)]
    pub(super) fn print_var(name: &str) -> Command {
        shell(&format!("echo \"${{{name}-unset}}\""))
    }

    #[cfg(windows)]
    pub(super) fn print_var(name: &str) -> Command {
        let mut command = Command::new("cmd");
        std::os::windows::process::CommandExt::raw_arg(
            command.args(["/V:ON", "/C"]),
            format!("if defined {name} (echo(!{name}!) else (echo unset)"),
        );
        command
    }

    pub(super) fn process(current_dir: PathBuf) -> ProcessImpl {
        ProcessImpl::new(current_dir)
    }

    /// Records the runs it handles and the output passing through it.
    #[derive(Clone, Default)]
    pub(super) struct Recorder(pub(super) Arc<std::sync::Mutex<Recorded>>);

    #[derive(Default)]
    pub(super) struct Recorded {
        pub(super) events: Vec<String>,
        pub(super) options: Vec<CommandOptions>,
        pub(super) stdout: Vec<u8>,
        pub(super) stderr: Vec<u8>,
    }

    impl Recorder {
        pub(super) fn recorded(&self) -> std::sync::MutexGuard<'_, Recorded> {
            self.0.lock().unwrap()
        }

        pub(super) fn events(&self) -> Vec<String> {
            self.recorded().events.clone()
        }

        pub(super) fn options(&self) -> CommandOptions {
            self.recorded().options[0].clone()
        }

        pub(super) fn stdout(&self) -> Vec<u8> {
            self.recorded().stdout.clone()
        }

        pub(super) fn stderr(&self) -> Vec<u8> {
            self.recorded().stderr.clone()
        }
    }

    impl Interceptor<ProcessCommand> for Recorder {
        fn handle<'a>(
            &'a self,
            mut next: Next<'a, ProcessCommand>,
        ) -> BoxFuture<'a, Result<CommandOutput>> {
            Box::pin(async move {
                {
                    let command = next.run().subject();
                    let mut recorded = self.recorded();
                    recorded.events.push(format!(
                        "started in {}",
                        command.current_dir().unwrap().display()
                    ));
                    recorded.options.push(command.options().clone());
                }

                let result = next.attempt().await;
                self.recorded().events.push(match &result {
                    Ok(output) if output.success => "finished ok".to_string(),
                    Ok(output) => format!("finished failed {:?}", output.status),
                    Err(error) => format!("finished {error}"),
                });
                result
            })
        }

        fn chunk(&self, chunk: Chunk<ProcessCommand>) -> Result {
            match chunk.stream() {
                OutputStream::Stdout => self.recorded().stdout.extend_from_slice(chunk.data()),
                OutputStream::Stderr => self.recorded().stderr.extend_from_slice(chunk.data()),
            }

            chunk.forward()
        }

        fn cancelled(&self, run: &Run<ProcessCommand>) {
            self.recorded()
                .events
                .push(format!("cancelled {}", run.subject().program()));
        }
    }

    /// Calls the closure with the run before passing it on.
    pub(super) struct Inspect<F>(pub(super) F);

    impl<F: Fn(&mut Run<ProcessCommand>) + Send + Sync + 'static> Interceptor<ProcessCommand>
        for Inspect<F>
    {
        fn handle<'a>(
            &'a self,
            mut next: Next<'a, ProcessCommand>,
        ) -> BoxFuture<'a, Result<CommandOutput>> {
            (self.0)(next.run_mut());
            next.forward()
        }
    }

    /// Accepts the outputs with the exit code.
    pub(super) struct Accept(pub(super) i32);

    impl Interceptor<ProcessCommand> for Accept {
        fn handle<'a>(
            &'a self,
            mut next: Next<'a, ProcessCommand>,
        ) -> BoxFuture<'a, Result<CommandOutput>> {
            Box::pin(async move {
                next.attempt().await.map(|output| CommandOutput {
                    success: output.success || output.status == Some(self.0),
                    ..output
                })
            })
        }
    }

    /// Sets the replay output of the runs.
    pub(super) struct Replay(pub(super) CommandOutput);

    impl Interceptor<ProcessCommand> for Replay {
        fn handle<'a>(
            &'a self,
            mut next: Next<'a, ProcessCommand>,
        ) -> BoxFuture<'a, Result<CommandOutput>> {
            next.set_replay_output(self.0.clone());
            next.forward()
        }
    }

    /// Changes of the env the process shares, like by its platform.
    impl ProcessImpl {
        pub(super) fn set_var(&self, key: String, value: String) {
            self.env.set_var(key, value).unwrap();
        }

        pub(super) fn remove_var(&self, key: &str) {
            self.env.remove_var(key);
        }

        pub(super) fn set_current_dir(&self, path: PathBuf) {
            self.env.replace_current_dir(path);
        }
    }

    /// Returns a process in the temporary directory whose only interceptor
    /// is the returned recorder.
    pub(super) fn observed() -> (ProcessImpl, Recorder) {
        let recorder = Recorder::default();
        let process = process(std::env::temp_dir());
        process.clear_interceptors();
        process.add_interceptor(recorder.clone());
        (process, recorder)
    }

    /// Returns the interceptors `retry` and `recorder`, in that order.
    pub(super) fn retried(
        retry: Retry<ProcessCommand>,
        recorder: &Recorder,
    ) -> Interceptors<ProcessCommand> {
        let mut interceptors = Interceptors::new();
        interceptors.push(retry);
        interceptors.push(recorder.clone());
        interceptors
    }

    /// Returns the run of `command` resolved against the env of `process`,
    /// as when it starts.
    pub(super) fn resolved(command: &Command, process: &ProcessImpl) -> Run<ProcessCommand> {
        let mut run = Run::from(command);
        run.subject_mut().resolve(&process.env);
        run
    }

    /// Returns the command of `command` with `options`.
    pub(super) fn with_options(command: &Command, options: CommandOptions) -> ProcessCommand {
        ProcessCommand::with_options(command, options)
    }

    /// Returns an output with the exit code `status`.
    pub(super) fn exit(status: i32) -> CommandOutput {
        CommandOutput {
            status: Some(status),
            ..Default::default()
        }
    }

    #[cfg(feature = "tokio")]
    pub(super) fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }

    mod command_ext {
        use super::*;

        mod run_blocking {
            use super::*;

            #[test]
            fn run_blocking_should_run_command_with_default_options() {
                let (process, recorder) = observed();

                shell("exit 0").run_blocking(&process).unwrap();

                assert_eq!(recorder.options(), CommandOptions::default());
            }
        }

        mod run_blocking_with_sink {
            use super::*;

            #[test]
            fn run_blocking_with_sink_should_run_command_with_default_options_to_sink() {
                let (process, recorder) = observed();
                let mut sunk = (Vec::new(), Vec::new());

                shell("echo out")
                    .run_blocking_with_sink(&process, &mut sink_into(&mut sunk))
                    .unwrap();

                assert_eq!(recorder.options(), CommandOptions::default());
                assert_eq!(String::from_utf8_lossy(&sunk.0).trim(), "out");
            }
        }

        #[cfg(feature = "tokio")]
        mod run_async {
            use super::*;

            #[test]
            fn run_async_should_run_command_with_default_options() {
                let (process, recorder) = observed();

                block_on(shell("exit 0").run_async(&process)).unwrap();

                assert_eq!(recorder.options(), CommandOptions::default());
            }
        }

        #[cfg(feature = "tokio")]
        mod run_async_with_sink {
            use super::*;

            #[test]
            fn run_async_with_sink_should_run_command_with_default_options_to_sink() {
                let (process, recorder) = observed();
                let mut sunk = (Vec::new(), Vec::new());

                block_on(
                    shell("echo out").run_async_with_sink(&process, &mut sink_into(&mut sunk)),
                )
                .unwrap();

                assert_eq!(recorder.options(), CommandOptions::default());
                assert_eq!(String::from_utf8_lossy(&sunk.0).trim(), "out");
            }
        }

        mod clear_env {
            use super::*;

            #[test]
            fn clear_env_should_run_command_without_configured_vars() {
                let (process, recorder) = observed();

                shell("exit 0").clear_env().run_blocking(&process).unwrap();

                assert_eq!(
                    recorder.options(),
                    CommandOptions {
                        clear_env: true,
                        ..Default::default()
                    }
                );
            }
        }

        mod display_lines {
            use super::*;

            #[test]
            fn display_lines_should_limit_lines_of_displayed_output() {
                let (process, recorder) = observed();

                let error = shell("echo 1&& echo 2&& echo 3&& exit 1")
                    .display_lines(1)
                    .run_blocking(&process)
                    .unwrap_err();

                assert_eq!(recorder.options().display_lines, 1);
                let output = error.command_output().unwrap();
                assert_eq!(output.display_lines, 1);
                assert!(output.to_string().contains("stdout:\n(2 lines omitted)\n3"));
            }
        }

        mod input {
            use super::*;

            #[test]
            fn input_should_run_command_with_input() {
                let (process, recorder) = observed();

                shell("exit 0")
                    .input("b\na")
                    .run_blocking(&process)
                    .unwrap();

                assert_eq!(
                    recorder.options(),
                    CommandOptions {
                        input: Some(b"b\na".to_vec()),
                        ..Default::default()
                    }
                );
            }
        }

        mod interactive {
            use super::*;

            #[test]
            fn interactive_should_run_command_interactively() {
                let (process, recorder) = observed();

                shell("exit 0")
                    .interactive()
                    .run_blocking(&process)
                    .unwrap();

                assert_eq!(
                    recorder.options(),
                    CommandOptions {
                        interactive: true,
                        ..Default::default()
                    }
                );
            }
        }

        mod silent {
            use super::*;

            #[test]
            fn silent_should_run_command_discarding_output() {
                let (process, recorder) = observed();

                shell("exit 0").silent().run_blocking(&process).unwrap();

                assert_eq!(
                    recorder.options(),
                    CommandOptions {
                        silent: true,
                        ..Default::default()
                    }
                );
            }
        }

        mod stream {
            use super::*;

            #[test]
            fn stream_should_run_command_streaming_output() {
                let (process, recorder) = observed();

                shell("exit 0").stream().run_blocking(&process).unwrap();

                assert_eq!(
                    recorder.options(),
                    CommandOptions {
                        stream: true,
                        ..Default::default()
                    }
                );
            }
        }

        mod timeout {
            use super::*;

            #[test]
            fn timeout_should_run_command_with_timeout() {
                let (process, recorder) = observed();

                shell("exit 0")
                    .timeout(Duration::from_secs(10))
                    .run_blocking(&process)
                    .unwrap();

                assert_eq!(
                    recorder.options(),
                    CommandOptions {
                        timeout: Some(Duration::from_secs(10)),
                        ..Default::default()
                    }
                );
            }
        }

        mod interceptors {
            use super::*;

            #[test]
            fn interceptors_should_run_command_with_own_interceptors_only() {
                let (process, recorder) = observed();
                let own = Recorder::default();
                let mut interceptors = Interceptors::new();
                interceptors.push(own.clone());

                shell("exit 0")
                    .interceptors(interceptors)
                    .run_blocking(&process)
                    .unwrap();

                assert!(recorder.events().is_empty());
                assert_eq!(own.events().len(), 2);
            }
        }

        mod with {
            use super::*;

            #[test]
            fn with_should_set_value_for_interceptors() {
                let seen = Arc::new(std::sync::Mutex::new(None));
                let process = process(std::env::temp_dir());
                let found = seen.clone();
                process.add_interceptor(Inspect(move |run: &mut Run<ProcessCommand>| {
                    *found.lock().unwrap() = run.get::<u32>().copied();
                }));

                shell("exit 0").with(7_u32).run_blocking(&process).unwrap();

                assert_eq!(*seen.lock().unwrap(), Some(7));
            }
        }
    }

    mod failure {
        use super::*;

        #[test]
        fn failure_should_return_error_with_output() {
            for (status, signal, timed_out, description) in [
                (Some(1), None, false, "'git' failed with exit code 1"),
                (
                    Some(0),
                    None,
                    false,
                    "'git' was not accepted with exit code 0",
                ),
                (
                    None,
                    None,
                    false,
                    "'git' was terminated without an exit code",
                ),
                (None, Some(9), false, "'git' was terminated by signal 9"),
                (None, Some(9), true, "'git' timed out after 1s"),
            ] {
                let output = CommandOutput {
                    status,
                    signal,
                    timed_out,
                    duration: Duration::from_secs(1),
                    ..Default::default()
                };

                let error = failure(&ProcessCommand::new(&Command::new("git")), output.clone());

                assert_eq!(error.description(), description);
                assert_eq!(error.command_output(), Some(&output), "{description}");
                assert_eq!(error.program(), Some("git"), "{description}");
            }
        }
    }

    mod io_error {
        use super::*;

        #[test]
        fn io_error_should_return_error_with_program() {
            let error = io_error("git", std::io::Error::from(std::io::ErrorKind::NotFound));

            assert_eq!(error.program(), Some("git"));
            assert!(error.source_as::<std::io::Error>().is_some());
        }
    }

    mod millis_until {
        use super::*;

        #[test]
        fn millis_until_should_round_up_remaining_time() {
            let millis = millis_until(Instant::now() + Duration::from_micros(10_500));

            assert!((10..=11).contains(&millis), "millis: {millis}");
        }

        #[test]
        fn millis_until_passed_deadline_should_return_zero() {
            assert_eq!(millis_until(Instant::now()), 0);
        }

        #[test]
        fn millis_until_distant_deadline_should_saturate() {
            let deadline = Instant::now() + Duration::from_secs(365 * 24 * 3600);

            assert_eq!(millis_until(deadline), i32::MAX as u32);
        }
    }

    mod run_blocking {
        use super::*;

        #[test]
        fn run_blocking_failed_output_should_return_error_after_chain() {
            let (process, recorder) = observed();

            let error = process.run_blocking(&shell("exit 3")).unwrap_err();

            assert_eq!(recorder.events()[1], "finished failed Some(3)");
            assert_eq!(
                error.description(),
                format!("'{SHELL}' failed with exit code 3")
            );
        }
    }

    mod outputs {
        use super::*;

        mod push {
            use super::*;

            #[test]
            fn push_should_pass_chunks_on() {
                let mut chunks = Vec::new();
                let mut outputs = Outputs::new(|stream, data| {
                    chunks.push((stream, data.to_vec()));
                    Ok(())
                });

                outputs.push(OutputStream::Stdout, b"out");
                outputs.push(OutputStream::Stderr, b"err");

                assert!(!outputs.failed());
                outputs.finish().unwrap();
                assert_eq!(
                    chunks,
                    [
                        (OutputStream::Stdout, b"out".to_vec()),
                        (OutputStream::Stderr, b"err".to_vec())
                    ]
                );
            }

            #[test]
            fn push_failing_to_pass_on_should_fail_outputs() {
                let mut outputs = Outputs::new(|_, _| Err(crate::Error::process("full")));

                outputs.push(OutputStream::Stdout, b"out");

                assert!(outputs.failed());
                assert_eq!(outputs.finish().unwrap_err().description(), "full");
            }

            #[test]
            fn push_after_failing_to_pass_on_should_not_pass_chunks_on() {
                // GIVEN outputs whose first chunk failed to be passed on
                let mut passed = Vec::new();
                let mut outputs = Outputs::new(|stream, _| {
                    passed.push(stream);
                    Err(crate::Error::process(format!("{stream:?} failed")))
                });
                outputs.push(OutputStream::Stdout, b"out");

                // WHEN pushing another chunk
                outputs.push(OutputStream::Stderr, b"err");

                // THEN it is not passed on and the first error is kept
                assert_eq!(outputs.finish().unwrap_err().description(), "Stdout failed");
                assert_eq!(passed, [OutputStream::Stdout]);
            }
        }
    }

    pub(crate) mod process {
        use super::*;

        fn program(command: &Command) -> String {
            command.get_program().to_string_lossy().into_owned()
        }

        pub(crate) mod run_blocking {
            use super::*;

            /// `command` prints `out` and `err` and exits with code 0.
            pub(crate) fn run_blocking_should_capture_output(
                process: &impl Process,
                command: &Command,
            ) {
                let output = process.run_blocking(command).unwrap();

                assert!(output.success);
                assert_eq!(output.status, Some(0));
                assert_eq!(output.stdout_str().unwrap().trim(), "out");
                assert_eq!(output.stderr_str().unwrap().trim(), "err");
                assert!(!output.timed_out);
            }

            /// `command` prints `out` and exits with code 3.
            pub(crate) fn run_blocking_non_zero_exit_should_return_error_with_output(
                process: &impl Process,
                command: &Command,
            ) {
                let error = process.run_blocking(command).unwrap_err();

                let output = error.command_output().unwrap();
                assert_eq!(error.kind(), crate::ErrorKind::Process);
                assert_eq!(
                    error.description(),
                    format!("'{}' failed with exit code 3", program(command))
                );
                assert_eq!(error.program(), Some(program(command).as_str()));
                assert!(!output.success);
                assert_eq!(output.status, Some(3));
                assert_eq!(output.stdout_str().unwrap().trim(), "out");
            }

            /// `command` is killed by `SIGKILL`.
            #[cfg(unix)]
            pub(crate) fn run_blocking_killed_by_signal_should_report_signal(
                process: &impl Process,
                command: &Command,
            ) {
                let error = process.run_blocking(command).unwrap_err();

                let output = error.command_output().unwrap();
                assert_eq!(
                    error.description(),
                    format!("'{}' was terminated by signal 9", program(command))
                );
                assert_eq!(output.status, None);
                assert_eq!(output.signal, Some(9));
                assert!(!output.timed_out);
            }

            /// `command` exits with code 0.
            pub(crate) fn run_blocking_invalid_options_should_return_error(
                process: &impl Process,
                command: &Command,
            ) {
                for (run, combined) in [
                    (command.interactive().input("in"), "input"),
                    (command.interactive().silent(), "silent"),
                    (
                        command.interactive().timeout(Duration::from_secs(1)),
                        "timeout",
                    ),
                ] {
                    let error = process.run_blocking(run).unwrap_err();

                    assert_eq!(
                        error.description(),
                        format!(
                            "'{}': interactive cannot be combined with {combined}",
                            program(command)
                        )
                    );
                }
            }

            /// `command` runs a program that is not found.
            pub(crate) fn run_blocking_missing_program_should_return_error_with_io_source(
                process: &impl Process,
                command: &Command,
            ) {
                let error = process.run_blocking(command).unwrap_err();

                assert_eq!(error.kind(), crate::ErrorKind::Process);
                assert!(
                    error
                        .description()
                        .starts_with(&format!("'{}': ", program(command))),
                    "{error}"
                );
                assert_eq!(error.program(), Some(program(command).as_str()));
                assert_eq!(
                    error
                        .source_as::<std::io::Error>()
                        .map(std::io::Error::kind),
                    Some(std::io::ErrorKind::NotFound)
                );
                assert!(error.command_output().is_none());
            }

            /// `command` prints `out` and exits with code 0 within 10 s.
            pub(crate) fn run_blocking_within_timeout_should_succeed(
                process: &impl Process,
                command: &Command,
            ) {
                let output = process
                    .run_blocking(command.timeout(Duration::from_secs(10)))
                    .unwrap();

                assert!(output.success);
                assert_eq!(output.stdout_str().unwrap().trim(), "out");
                assert!(!output.timed_out);
            }

            /// `command` prints `out` and exits with code 0.
            pub(crate) fn run_blocking_maximum_timeout_should_not_limit_run(
                process: &impl Process,
                command: &Command,
            ) {
                for timeout in [Duration::MAX, Duration::from_secs(1000 * 365 * 24 * 3600)] {
                    let output = process.run_blocking(command.timeout(timeout)).unwrap();

                    assert_eq!(output.stdout_str().unwrap().trim(), "out", "{timeout:?}");
                }
            }

            /// `command` prints `out` and exits with code 0.
            pub(crate) fn run_blocking_zero_timeout_should_time_out(
                process: &impl Process,
                command: &Command,
            ) {
                let error = process
                    .run_blocking(command.timeout(Duration::ZERO))
                    .unwrap_err();

                let output = error.command_output().unwrap();
                assert!(output.timed_out);
                assert!(!output.success);
                assert_eq!(output.status, None);
            }

            /// `command` prints `out` and then runs for longer than
            /// `timeout`.
            pub(crate) fn run_blocking_exceeding_timeout_should_time_out_keeping_output(
                process: &impl Process,
                command: &Command,
                timeout: Duration,
            ) {
                let error = process.run_blocking(command.timeout(timeout)).unwrap_err();

                let output = error.command_output().unwrap();
                assert!(
                    error
                        .description()
                        .starts_with(&format!("'{}' timed out after ", program(command))),
                    "{error}"
                );
                assert!(output.timed_out);
                assert!(!output.success);
                assert_eq!(output.status, None);
                assert!(output.duration >= timeout, "{:?}", output.duration);
                assert_eq!(output.stdout_str().unwrap().trim(), "out");
            }

            /// `command` prints `out` and `err` and exits with code 0.
            pub(crate) fn run_blocking_silent_should_neither_capture_nor_stream_output(
                process: &impl Process,
                command: &Command,
            ) {
                let recorder = Recorder::default();
                process.add_interceptor(recorder.clone());

                let output = process.run_blocking(command.stream().silent()).unwrap();

                assert_eq!(output.status, Some(0));
                assert!(output.stdout.is_empty());
                assert!(output.stderr.is_empty());
                assert!(recorder.stdout().is_empty());
                assert!(recorder.stderr().is_empty());
            }

            /// `command` exits with code 3, printing only to the terminal if
            /// at all.
            pub(crate) fn run_blocking_interactive_should_not_capture_output(
                process: &impl Process,
                command: &Command,
            ) {
                let error = process.run_blocking(command.interactive()).unwrap_err();

                let output = error.command_output().unwrap();
                assert_eq!(output.status, Some(3));
                assert!(output.stdout.is_empty());
                assert!(output.stderr.is_empty());
            }

            /// `command` prints `out` and `err` and exits with code 0.
            pub(crate) fn run_blocking_streaming_should_pass_output_to_interceptors(
                process: &impl Process,
                command: &Command,
            ) {
                let recorder = Recorder::default();
                process.add_interceptor(recorder.clone());

                let output = process.run_blocking(command.stream()).unwrap();

                assert_eq!(recorder.stdout(), output.stdout);
                assert_eq!(recorder.stderr(), output.stderr);
                assert_eq!(output.stdout_str().unwrap().trim(), "out");
            }

            /// `succeeding` exits with code 0 and `failing` with code 3,
            /// both in `dir`, the current directory of `process`.
            pub(crate) fn run_blocking_should_pass_run_and_result_to_interceptors(
                process: &impl Process,
                succeeding: &Command,
                failing: &Command,
                dir: &Path,
            ) {
                let recorder = Recorder::default();
                process.clear_interceptors();
                process.add_interceptor(recorder.clone());

                process.run_blocking(succeeding).unwrap();
                let _ = process.run_blocking(failing);

                assert_eq!(
                    recorder.events(),
                    [
                        format!("started in {}", dir.display()),
                        "finished ok".to_string(),
                        format!("started in {}", dir.display()),
                        "finished failed Some(3)".to_string()
                    ]
                );
            }
        }

        pub(crate) mod run_blocking_with_sink {
            use super::*;

            /// `command` prints `out` and `err` and exits with code 0.
            pub(crate) fn run_blocking_with_sink_should_pass_output_to_sink_instead_of_capturing(
                process: &impl Process,
                command: &Command,
            ) {
                let mut sunk = (Vec::new(), Vec::new());

                let output = process
                    .run_blocking_with_sink(command, &mut sink_into(&mut sunk))
                    .unwrap();

                assert_eq!(output.status, Some(0));
                assert!(output.stdout.is_empty() && output.stderr.is_empty());
                assert_eq!(String::from_utf8_lossy(&sunk.0).trim(), "out");
                assert_eq!(String::from_utf8_lossy(&sunk.1).trim(), "err");
            }

            /// `command` prints `out` and exits with code 0.
            pub(crate) fn run_blocking_with_sink_should_pass_output_through_interceptors_to_sink(
                process: &impl Process,
                command: &Command,
            ) {
                let recorder = Recorder::default();
                process.add_interceptor(recorder.clone());
                let mut sunk = (Vec::new(), Vec::new());

                process
                    .run_blocking_with_sink(command.stream(), &mut sink_into(&mut sunk))
                    .unwrap();

                assert_eq!(recorder.stdout(), sunk.0);
                assert_eq!(String::from_utf8_lossy(&sunk.0).trim(), "out");
            }

            /// `command` prints `out` and `err` and exits with code 3.
            pub(crate) fn run_blocking_with_sink_failed_should_return_error_without_output(
                process: &impl Process,
                command: &Command,
            ) {
                let mut sunk = (Vec::new(), Vec::new());

                let error = process
                    .run_blocking_with_sink(command, &mut sink_into(&mut sunk))
                    .unwrap_err();

                let output = error.command_output().unwrap();
                assert_eq!(output.status, Some(3));
                assert!(output.stdout.is_empty() && output.stderr.is_empty());
                assert_eq!(String::from_utf8_lossy(&sunk.0).trim(), "out");
                assert_eq!(String::from_utf8_lossy(&sunk.1).trim(), "err");
            }

            /// `command` prints `out` and `err` and exits with code 0.
            pub(crate) fn run_blocking_with_sink_failing_sink_should_return_its_error(
                process: &impl Process,
                command: &Command,
            ) {
                for failing in [OutputStream::Stdout, OutputStream::Stderr] {
                    let mut streams = Vec::new();

                    let error = process
                        .run_blocking_with_sink(command, &mut |stream: OutputStream, _: &[u8]| {
                            streams.push(stream);
                            (stream != failing)
                                .then_some(())
                                .ok_or_else(|| crate::Error::process("full"))
                        })
                        .unwrap_err();

                    assert_eq!(error.description(), "full", "{failing:?}");
                    assert!(streams.contains(&failing), "{failing:?}");
                }
            }

            /// `command` prints `out` and exits with code 0.
            pub(crate) fn run_blocking_with_sink_silent_should_fail_without_running(
                process: &impl Process,
                command: &Command,
            ) {
                let mut sunk = (Vec::new(), Vec::new());

                let error = process
                    .run_blocking_with_sink(command.silent(), &mut sink_into(&mut sunk))
                    .unwrap_err();

                assert_eq!(
                    error.description(),
                    format!(
                        "'{}': silent cannot be combined with a sink",
                        program(command)
                    )
                );
                assert!(sunk.0.is_empty() && sunk.1.is_empty());
            }
        }

        #[cfg(feature = "tokio")]
        pub(crate) mod run_async {
            use super::*;

            /// `command` prints `out` and `err` and exits with code 0.
            pub(crate) fn run_async_should_capture_output(
                process: &impl Process,
                command: &Command,
            ) {
                let output = block_on(process.run_async(command)).unwrap();

                assert!(output.success);
                assert_eq!(output.status, Some(0));
                assert_eq!(output.stdout_str().unwrap().trim(), "out");
                assert_eq!(output.stderr_str().unwrap().trim(), "err");
                assert!(!output.timed_out);
            }

            /// `command` prints `out` and exits with code 3.
            pub(crate) fn run_async_non_zero_exit_should_return_error_with_output(
                process: &impl Process,
                command: &Command,
            ) {
                let error = block_on(process.run_async(command)).unwrap_err();

                let output = error.command_output().unwrap();
                assert_eq!(
                    error.description(),
                    format!("'{}' failed with exit code 3", program(command))
                );
                assert_eq!(error.program(), Some(program(command).as_str()));
                assert!(!output.success);
                assert_eq!(output.status, Some(3));
                assert_eq!(output.stdout_str().unwrap().trim(), "out");
            }

            /// `command` is killed by `SIGKILL`.
            #[cfg(unix)]
            pub(crate) fn run_async_killed_by_signal_should_report_signal(
                process: &impl Process,
                command: &Command,
            ) {
                let error = block_on(process.run_async(command)).unwrap_err();

                let output = error.command_output().unwrap();
                assert_eq!(
                    error.description(),
                    format!("'{}' was terminated by signal 9", program(command))
                );
                assert_eq!(output.status, None);
                assert_eq!(output.signal, Some(9));
                assert!(!output.timed_out);
            }

            /// `command` exits with code 0.
            pub(crate) fn run_async_invalid_options_should_return_error(
                process: &impl Process,
                command: &Command,
            ) {
                for (run, combined) in [
                    (command.interactive().input("in"), "input"),
                    (command.interactive().silent(), "silent"),
                    (
                        command.interactive().timeout(Duration::from_secs(1)),
                        "timeout",
                    ),
                ] {
                    let error = block_on(process.run_async(run)).unwrap_err();

                    assert_eq!(
                        error.description(),
                        format!(
                            "'{}': interactive cannot be combined with {combined}",
                            program(command)
                        )
                    );
                }
            }

            /// `command` runs a program that is not found.
            pub(crate) fn run_async_missing_program_should_return_error_with_io_source(
                process: &impl Process,
                command: &Command,
            ) {
                let error = block_on(process.run_async(command)).unwrap_err();

                assert!(
                    error
                        .description()
                        .starts_with(&format!("'{}': ", program(command))),
                    "{error}"
                );
                assert_eq!(error.program(), Some(program(command).as_str()));
                assert_eq!(
                    error
                        .source_as::<std::io::Error>()
                        .map(std::io::Error::kind),
                    Some(std::io::ErrorKind::NotFound)
                );
                assert!(error.command_output().is_none());
            }

            /// `command` prints `out` and exits with code 0 within 10 s.
            pub(crate) fn run_async_within_timeout_should_succeed(
                process: &impl Process,
                command: &Command,
            ) {
                let output =
                    block_on(process.run_async(command.timeout(Duration::from_secs(10)))).unwrap();

                assert!(output.success);
                assert_eq!(output.stdout_str().unwrap().trim(), "out");
                assert!(!output.timed_out);
            }

            /// `command` prints `out` and exits with code 0.
            pub(crate) fn run_async_maximum_timeout_should_not_limit_run(
                process: &impl Process,
                command: &Command,
            ) {
                for timeout in [Duration::MAX, Duration::from_secs(1000 * 365 * 24 * 3600)] {
                    let output = block_on(process.run_async(command.timeout(timeout))).unwrap();

                    assert_eq!(output.stdout_str().unwrap().trim(), "out", "{timeout:?}");
                }
            }

            /// `command` prints `out` and exits with code 0.
            pub(crate) fn run_async_zero_timeout_should_time_out(
                process: &impl Process,
                command: &Command,
            ) {
                let error =
                    block_on(process.run_async(command.timeout(Duration::ZERO))).unwrap_err();

                let output = error.command_output().unwrap();
                assert!(output.timed_out);
                assert!(!output.success);
                assert_eq!(output.status, None);
            }

            /// `command` prints `out` and then runs for longer than
            /// `timeout`.
            pub(crate) fn run_async_exceeding_timeout_should_time_out_keeping_output(
                process: &impl Process,
                command: &Command,
                timeout: Duration,
            ) {
                let error = block_on(process.run_async(command.timeout(timeout))).unwrap_err();

                let output = error.command_output().unwrap();
                assert!(
                    error
                        .description()
                        .starts_with(&format!("'{}' timed out after ", program(command))),
                    "{error}"
                );
                assert!(output.timed_out);
                assert!(!output.success);
                assert_eq!(output.status, None);
                assert!(output.duration >= timeout, "{:?}", output.duration);
                assert_eq!(output.stdout_str().unwrap().trim(), "out");
            }

            /// `command` prints `out` and `err` and exits with code 0.
            pub(crate) fn run_async_silent_should_neither_capture_nor_stream_output(
                process: &impl Process,
                command: &Command,
            ) {
                let recorder = Recorder::default();
                process.add_interceptor(recorder.clone());

                let output = block_on(process.run_async(command.stream().silent())).unwrap();

                assert_eq!(output.status, Some(0));
                assert!(output.stdout.is_empty());
                assert!(output.stderr.is_empty());
                assert!(recorder.stdout().is_empty());
                assert!(recorder.stderr().is_empty());
            }

            /// `command` exits with code 3, printing only to the terminal if
            /// at all.
            pub(crate) fn run_async_interactive_should_not_capture_output(
                process: &impl Process,
                command: &Command,
            ) {
                let error = block_on(process.run_async(command.interactive())).unwrap_err();

                let output = error.command_output().unwrap();
                assert_eq!(output.status, Some(3));
                assert!(output.stdout.is_empty());
                assert!(output.stderr.is_empty());
            }

            /// `command` prints `out` and `err` and exits with code 0.
            pub(crate) fn run_async_streaming_should_pass_output_to_interceptors(
                process: &impl Process,
                command: &Command,
            ) {
                let recorder = Recorder::default();
                process.add_interceptor(recorder.clone());

                let output = block_on(process.run_async(command.stream())).unwrap();

                assert_eq!(recorder.stdout(), output.stdout);
                assert_eq!(recorder.stderr(), output.stderr);
                assert_eq!(output.stdout_str().unwrap().trim(), "out");
            }

            /// `command` runs for longer than 200 ms.
            pub(crate) fn run_async_cancelled_should_tell_interceptors(
                process: &impl Process,
                command: &Command,
            ) {
                let recorder = Recorder::default();
                process.add_interceptor(recorder.clone());

                let result = block_on(async {
                    tokio::time::timeout(Duration::from_millis(200), process.run_async(command))
                        .await
                });

                assert!(result.is_err());
                assert_eq!(
                    recorder.events().last(),
                    Some(&format!("cancelled {}", program(command)))
                );
            }
        }

        #[cfg(feature = "tokio")]
        pub(crate) mod run_async_with_sink {
            use super::*;

            /// `command` prints `out` and `err` and exits with code 0.
            pub(crate) fn run_async_with_sink_should_pass_output_to_sink_instead_of_capturing(
                process: &impl Process,
                command: &Command,
            ) {
                let mut sunk = (Vec::new(), Vec::new());

                let output =
                    block_on(process.run_async_with_sink(command, &mut sink_into(&mut sunk)))
                        .unwrap();

                assert_eq!(output.status, Some(0));
                assert!(output.stdout.is_empty() && output.stderr.is_empty());
                assert_eq!(String::from_utf8_lossy(&sunk.0).trim(), "out");
                assert_eq!(String::from_utf8_lossy(&sunk.1).trim(), "err");
            }

            /// `command` prints `out` and `err` and exits with code 3.
            pub(crate) fn run_async_with_sink_failed_should_return_error_without_output(
                process: &impl Process,
                command: &Command,
            ) {
                let mut sunk = (Vec::new(), Vec::new());

                let error =
                    block_on(process.run_async_with_sink(command, &mut sink_into(&mut sunk)))
                        .unwrap_err();

                let output = error.command_output().unwrap();
                assert_eq!(output.status, Some(3));
                assert!(output.stdout.is_empty() && output.stderr.is_empty());
                assert_eq!(String::from_utf8_lossy(&sunk.0).trim(), "out");
                assert_eq!(String::from_utf8_lossy(&sunk.1).trim(), "err");
            }

            /// `command` prints `out` and `err` and exits with code 0.
            pub(crate) fn run_async_with_sink_failing_sink_should_return_its_error(
                process: &impl Process,
                command: &Command,
            ) {
                for failing in [OutputStream::Stdout, OutputStream::Stderr] {
                    let mut streams = Vec::new();

                    let error = block_on(process.run_async_with_sink(
                        command,
                        &mut |stream: OutputStream, _: &[u8]| {
                            streams.push(stream);
                            (stream != failing)
                                .then_some(())
                                .ok_or_else(|| crate::Error::process("full"))
                        },
                    ))
                    .unwrap_err();

                    assert_eq!(error.description(), "full", "{failing:?}");
                    assert!(streams.contains(&failing), "{failing:?}");
                }
            }
        }

        pub(crate) mod interceptors {
            use super::*;

            pub(crate) fn interceptors_should_return_std_stream_by_default(process: &impl Process) {
                let interceptors = process.interceptors();

                assert_eq!(interceptors.len(), 1);
                assert_eq!(interceptors, process.interceptors());
            }
        }

        pub(crate) mod clear_interceptors {
            use super::*;

            /// `command` exits with code 0.
            pub(crate) fn clear_interceptors_should_remove_interceptors_of_later_runs(
                process: &impl Process,
                command: &Command,
            ) {
                let recorder = Recorder::default();
                process.add_interceptor(recorder.clone());

                process.clear_interceptors();
                process.run_blocking(command).unwrap();

                assert!(process.interceptors().is_empty());
                assert!(recorder.events().is_empty());
            }
        }

        pub(crate) mod add_interceptor {
            use super::*;

            /// `command` exits with code 3.
            pub(crate) fn add_interceptor_should_add_innermost_interceptor(
                process: &impl Process,
                command: &Command,
            ) {
                // GIVEN a recorder after an interceptor accepting exit code 3
                let recorder = Recorder::default();
                process.clear_interceptors();
                process.add_interceptor(Accept(3));
                process.add_interceptor(recorder.clone());

                // WHEN running a command exiting with it
                let output = process.run_blocking(command).unwrap();

                // THEN the recorder sees the failure before it is accepted
                assert!(output.success);
                assert_eq!(recorder.events()[1], "finished failed Some(3)");
            }
        }
    }
}
