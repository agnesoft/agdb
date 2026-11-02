use std::borrow::Cow;
use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
#[cfg(feature = "tokio")]
use std::sync::Mutex;
#[cfg(feature = "tokio")]
use std::sync::PoisonError;
use std::time::Duration;
use std::time::Instant;

use crate::Result;
use crate::env::EnvImpl;
#[cfg(feature = "tokio")]
use crate::interceptor::AsyncExecutor;
use crate::interceptor::BlockingExecutor;
use crate::interceptor::BoxFuture;
use crate::interceptor::Emit;
use crate::interceptor::Executor;
use crate::interceptor::Interceptor;
use crate::interceptor::Interceptors;
use crate::interceptor::Run;
use crate::interceptor::Sink;
use crate::utils::Runs;

use super::CommandOutput;
use super::Outputs;
use super::Process;
use super::ProcessCommand;
use super::configure;
use super::exit_signal;
use super::io_error;
#[cfg(feature = "tokio")]
use super::run_async;
use super::run_blocking;
use super::std_runs;
use super::wait_with_output;

/// [`Process`] implementation backed by [`std::process`].
///
/// Child processes run in the configured current directory with the
/// configured environment variables.
///
/// Normally obtained via [`platform()`](crate::platform) rather than
/// constructed directly.
pub struct ProcessImpl {
    pub(super) env: EnvImpl,
    runs: Runs<ProcessCommand>,
}

/// [internal]
///
/// The [`Executor`] of the blocking runs of a [`ProcessImpl`].
pub(super) struct StdExecutor<'a>(pub(super) &'a ProcessImpl);

/// [internal]
///
/// The [`Executor`] of the async runs of a [`ProcessImpl`] on [`tokio`].
#[cfg(feature = "tokio")]
struct TokioExecutor<'a>(&'a ProcessImpl);

impl ProcessImpl {
    /// [internal]
    ///
    /// Creates a new `ProcessImpl` running child processes in
    /// `current_dir` with a copy of the variables of the current process.
    #[cfg(test)]
    pub(crate) fn new(current_dir: PathBuf) -> Self {
        Self::with_env(EnvImpl::new(std::env::vars_os().collect(), current_dir))
    }

    /// [internal]
    ///
    /// Creates a new `ProcessImpl` running child processes in the current
    /// directory of `env` with its variables.
    pub(crate) fn with_env(env: EnvImpl) -> Self {
        ProcessImpl {
            env,
            runs: std_runs(),
        }
    }

    /// Runs the command once with its `outputs`, blocking the thread, and
    /// returns its output without the standard output and error.
    fn attempt(&self, command: &ProcessCommand, mut outputs: Outputs) -> Result<CommandOutput> {
        let options = command.options();
        let start = Instant::now();
        let deadline = options
            .timeout
            .and_then(|timeout| start.checked_add(timeout));
        let result = self
            .child_command(command)
            .and_then(|mut child| {
                configure(&mut child, options.timeout.is_some());
                child.spawn()
            })
            .and_then(|mut child| {
                wait_with_output(
                    &mut child,
                    options.input.as_deref().unwrap_or_default(),
                    deadline,
                    &mut outputs,
                )
            });

        exited(command, start, result, outputs)
    }

    /// Runs the command once with its `outputs` like [`Self::attempt`]
    /// asynchronously.
    #[cfg(feature = "tokio")]
    async fn attempt_async(
        &self,
        command: &ProcessCommand,
        outputs: Outputs<'_>,
    ) -> Result<CommandOutput> {
        let outputs = Mutex::new(outputs);
        let options = command.options();
        let group = !options.interactive;
        let start = Instant::now();
        let deadline = options
            .timeout
            .and_then(|timeout| start.checked_add(timeout));
        let result = match self.child_command(command).and_then(|mut child| {
            configure(&mut child, group);
            tokio::process::Command::from(child)
                .kill_on_drop(true)
                .spawn()
        }) {
            Ok(mut child) => {
                super::tokio_process::wait_with_output(
                    &mut child,
                    group,
                    options.input.as_deref().unwrap_or_default(),
                    deadline,
                    &outputs,
                )
                .await
            }
            Err(error) => Err(error),
        };

        exited(
            command,
            start,
            result,
            outputs.into_inner().unwrap_or_else(PoisonError::into_inner),
        )
    }

    /// Returns a new command with the program, the arguments and the
    /// environment variables of `command` resolved against the configuration.
    /// The child gets exactly the variables of the platform, inherited while
    /// they are those of the current process, none with
    /// [`CommandOptions::clear_env`](crate::CommandOptions::clear_env), updated
    /// by those of the command.
    ///
    /// # Errors
    ///
    /// Returns an error if the program is not found, see
    /// [`Self::program`].
    fn child_command(&self, command: &ProcessCommand) -> std::io::Result<Command> {
        let options = command.options();
        // WHY: only commands resolved against the env are run.
        let dir = command.current_dir().unwrap_or(Path::new(""));
        #[cfg(unix)]
        let program = self.program(command, dir)?;
        #[cfg(windows)]
        let program = self.program(command, dir);
        let mut child = Command::new(program.as_os_str());
        #[cfg(unix)]
        std::os::unix::process::CommandExt::arg0(&mut child, &command.program);
        child.args(command.args()).current_dir(dir);

        let (stdin, stdout, stderr) = if options.interactive {
            (Stdio::inherit(), Stdio::inherit(), Stdio::inherit())
        } else if options.input.is_some() {
            (Stdio::piped(), Stdio::piped(), Stdio::piped())
        } else {
            (Stdio::null(), Stdio::piped(), Stdio::piped())
        };
        let (stdout, stderr) = if options.silent {
            (Stdio::null(), Stdio::null())
        } else {
            (stdout, stderr)
        };
        child.stdin(stdin).stdout(stdout).stderr(stderr);

        if options.clear_env {
            child.env_clear();
        } else if let Some(vars) = command.vars() {
            child.env_clear();
            child.envs(vars.iter().map(|(key, value)| (key, value)));
        }

        for (key, value) in command.envs() {
            match value {
                Some(value) => child.env(key, value),
                None => child.env_remove(key),
            };
        }

        Ok(child)
    }

    /// Returns the program of `command` as the child process running in
    /// `dir` finds it.
    ///
    /// WHY: std resolves a relative path with a separator against the
    /// current directory of the current process on Windows. On Unix it
    /// would look a bare name up in the `PATH` of the child process, which
    /// [`CommandOptions::clear_env`] removes, and only by forking when the
    /// environment of the child is set.
    #[cfg(unix)]
    fn program<'a>(
        &self,
        command: &'a ProcessCommand,
        dir: &Path,
    ) -> std::io::Result<Cow<'a, Path>> {
        let program = Path::new(&command.program);

        if program
            .as_os_str()
            .as_encoded_bytes()
            .iter()
            .any(|byte| std::path::is_separator(char::from(*byte)))
        {
            Ok(Cow::Owned(dir.join(program)))
        } else {
            self.search_path(command, program, dir)
        }
    }

    /// Returns the program of `command` as the child process running in
    /// `dir` finds it.
    ///
    /// WHY: std resolves a relative path with a separator against the
    /// current directory of the current process on Windows. A bare name is
    /// looked up by std from the child's `PATH`.
    #[cfg(windows)]
    fn program<'a>(&self, command: &'a ProcessCommand, dir: &Path) -> Cow<'a, Path> {
        let program = Path::new(&command.program);

        if program
            .as_os_str()
            .as_encoded_bytes()
            .iter()
            .any(|byte| std::path::is_separator(char::from(*byte)))
        {
            Cow::Owned(dir.join(program))
        } else {
            self.search_path(program)
        }
    }

    /// Returns the bare `program` for std to look up.
    ///
    /// WHY: std on Windows looks it up in the `PATH` of the child process,
    /// that of the platform, falling back to the one of the current process.
    #[cfg(windows)]
    fn search_path<'a>(&self, program: &'a Path) -> Cow<'a, Path> {
        let _ = self;
        Cow::Borrowed(program)
    }

    /// Returns the bare `program` of `command` found in its own `PATH`, or that
    /// of the platform otherwise, even with
    /// [`CommandOptions::clear_env`](crate::CommandOptions::clear_env). The
    /// bare `program` for std to look up when the command removes it or neither
    /// sets it.
    #[cfg(unix)]
    fn search_path<'a>(
        &self,
        command: &ProcessCommand,
        program: &'a Path,
        dir: &Path,
    ) -> std::io::Result<Cow<'a, Path>> {
        let _ = self;
        let path = match command.envs().iter().find(|(key, _)| key == "PATH") {
            Some((_, value)) => value.clone(),
            None => command.var(std::ffi::OsStr::new("PATH")),
        };

        match path {
            Some(path) => super::unix::find_program(program, &path, dir).map(Cow::Owned),
            None => Ok(Cow::Borrowed(program)),
        }
    }
}

impl Process for ProcessImpl {
    fn run_blocking(&self, run: impl Into<Run<ProcessCommand>>) -> Result<CommandOutput> {
        run_blocking(&self.runs, &self.env, &StdExecutor(self), run.into(), None)
    }

    fn run_blocking_with_sink(
        &self,
        run: impl Into<Run<ProcessCommand>>,
        sink: &mut dyn Sink<ProcessCommand>,
    ) -> Result<CommandOutput> {
        run_blocking(
            &self.runs,
            &self.env,
            &StdExecutor(self),
            run.into(),
            Some(sink),
        )
    }

    #[cfg(feature = "tokio")]
    fn run_async(
        &self,
        run: impl Into<Run<ProcessCommand>>,
    ) -> impl Future<Output = Result<CommandOutput>> + Send {
        let run = run.into();
        async move { run_async(&self.runs, &self.env, &TokioExecutor(self), run, None).await }
    }

    #[cfg(feature = "tokio")]
    fn run_async_with_sink(
        &self,
        run: impl Into<Run<ProcessCommand>>,
        sink: &mut dyn Sink<ProcessCommand>,
    ) -> impl Future<Output = Result<CommandOutput>> + Send {
        let run = run.into();
        async move { run_async(&self.runs, &self.env, &TokioExecutor(self), run, Some(sink)).await }
    }

    fn interceptors(&self) -> Interceptors<ProcessCommand> {
        self.runs.interceptors()
    }

    fn clear_interceptors(&self) {
        self.runs.clear_interceptors();
    }

    fn add_interceptor(&self, interceptor: impl Interceptor<ProcessCommand>) {
        self.runs.add_interceptor(interceptor);
    }
}

impl Executor for StdExecutor<'_> {
    type Subject = ProcessCommand;

    fn execute<'a>(
        &'a self,
        run: &'a Run<ProcessCommand>,
        mut emit: Emit<'a, ProcessCommand>,
    ) -> BoxFuture<'a, Result<CommandOutput>> {
        Box::pin(async move {
            let outputs = Outputs::new(move |stream, data| emitted(emit.emit(stream, data)));
            self.0.attempt(run.subject(), outputs)
        })
    }

    fn sleep(&self, delay: Duration) -> BoxFuture<'static, ()> {
        Box::pin(async move { std::thread::sleep(delay) })
    }
}

impl BlockingExecutor for StdExecutor<'_> {}

#[cfg(feature = "tokio")]
impl Executor for TokioExecutor<'_> {
    type Subject = ProcessCommand;

    fn execute<'a>(
        &'a self,
        run: &'a Run<ProcessCommand>,
        mut emit: Emit<'a, ProcessCommand>,
    ) -> BoxFuture<'a, Result<CommandOutput>> {
        Box::pin(async move {
            let outputs = Outputs::new(move |stream, data| emitted(emit.emit(stream, data)));
            self.0.attempt_async(run.subject(), outputs).await
        })
    }

    fn sleep(&self, delay: Duration) -> BoxFuture<'static, ()> {
        Box::pin(tokio::time::sleep(delay))
    }
}

#[cfg(feature = "tokio")]
impl AsyncExecutor for TokioExecutor<'_> {}

/// [internal]
///
/// Waits for passing a chunk of the output on with `future`, see
/// [`Emit::emit`], on the current thread.
///
/// WHY: the parts of the executors reading the output of a child process
/// are sync, so they cannot await it. It is ready at once, as the chunk
/// interceptors are sync and the sinks of the runs of commands are
/// closures, so it does not block a runtime worker in [`TokioExecutor`].
pub(crate) fn emitted(future: impl Future<Output = Result>) -> Result {
    let mut future = std::pin::pin!(future);
    let mut polled = false;

    crate::interceptor::block_on(std::future::poll_fn(|context| {
        debug_assert!(!polled, "passing a chunk of a command on should not wait");
        polled = true;
        future.as_mut().poll(context)
    }))
}

/// Returns the output of the attempt of `command` started at `start` from
/// the `result` of its child process, without the standard output and
/// error, which went to `outputs`.
fn exited(
    command: &ProcessCommand,
    start: Instant,
    result: std::io::Result<Option<std::process::ExitStatus>>,
    outputs: Outputs,
) -> Result<CommandOutput> {
    outputs.finish()?;
    let status = result.map_err(|error| io_error(&command.program(), error))?;

    Ok(CommandOutput {
        status: status.and_then(|status| status.code()),
        signal: status.as_ref().and_then(exit_signal),
        duration: start.elapsed(),
        timed_out: status.is_none(),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OutputStream;
    use crate::TestDir;
    use crate::interceptor::Retry;
    use crate::process::CommandExt;
    use crate::process::CommandOptions;
    use crate::process::tests::*;

    #[cfg(unix)]
    const ECHO_THEN_SLEEP: &str = "echo out; sleep 10";

    #[cfg(windows)]
    const ECHO_THEN_SLEEP: &str = "echo out& ping -n 11 127.0.0.1 >NUL";

    #[cfg(all(unix, feature = "tokio"))]
    const MARK_LATER_THEN_SLEEP: &str = "(sleep 0.5; touch marker) & sleep 10";

    #[cfg(all(windows, feature = "tokio"))]
    const MARK_LATER_THEN_SLEEP: &str = "start \"\" /b cmd /C \"ping -n 2 127.0.0.1 >NUL & type nul > marker\" & ping -n 11 127.0.0.1 >NUL";

    /// When [`MARK_LATER_THEN_SLEEP`] would have created the marker.
    #[cfg(all(unix, feature = "tokio"))]
    const AFTER_MARKER: Duration = Duration::from_millis(1000);

    #[cfg(all(windows, feature = "tokio"))]
    const AFTER_MARKER: Duration = Duration::from_millis(2000);

    /// Starts a descendant creating a marker after 0.5 s before printing.
    #[cfg(unix)]
    const MARK_LATER_THEN_ECHO_THEN_SLEEP: &str = "(sleep 0.5; touch marker) & echo out; sleep 10";

    /// Prints the pid of a sleeping descendant holding the output open.
    #[cfg(unix)]
    const BACKGROUND_SLEEPER: &str = "sleep 10 & echo $!";

    /// Prints 0.
    #[cfg(windows)]
    const BACKGROUND_SLEEPER: &str = "echo 0";

    #[cfg(unix)]
    const EXITED_CHILD_WAIT_MAX: Duration = Duration::from_secs(2);

    #[cfg(windows)]
    const EXITED_CHILD_WAIT_MAX: Duration = Duration::from_secs(6);

    /// Writes more than a pipe holds to the standard error, then to the
    /// standard output.
    #[cfg(unix)]
    const LARGE_OUTPUT_ON_BOTH_STREAMS: &str =
        "head -c 1000000 /dev/zero >&2; head -c 1000000 /dev/zero";

    #[cfg(unix)]
    const TOOL: &str = "agplatform_tool";

    #[cfg(windows)]
    const TOOL: &str = "agplatform_tool.cmd";

    #[cfg(unix)]
    const FAIL_ONCE: &str = "test -f marker || { touch marker; exit 1; }";

    #[cfg(windows)]
    const FAIL_ONCE: &str = "if exist marker (exit 0) else (type nul > marker & exit 1)";

    #[cfg(unix)]
    const PRINT_DIR: &str = "pwd";

    #[cfg(windows)]
    const PRINT_DIR: &str = "cd";

    #[cfg(unix)]
    fn echo_stdin() -> Command {
        Command::new("cat")
    }

    #[cfg(windows)]
    fn echo_stdin() -> Command {
        let mut command = Command::new("findstr");
        command.arg("^");
        command
    }

    #[cfg(unix)]
    fn kill_sleeper(pid: &str) {
        Command::new("kill").arg(pid.trim()).status().unwrap();
    }

    #[cfg(windows)]
    fn kill_sleeper(_pid: &str) {}

    /// Writes the program [`TOOL`] printing `tool` to `dir`.
    ///
    /// WHY: written by a child process, as the children forked meanwhile by
    /// other tests would inherit a descriptor writing it, failing its
    /// execution with ETXTBSY.
    #[cfg(unix)]
    fn write_tool(dir: &Path) {
        let status = shell("printf '#!/bin/sh\\necho tool\\n' > \"$0\" && chmod 755 \"$0\"")
            .arg(dir.join(TOOL))
            .status()
            .unwrap();

        assert!(status.success());
    }

    #[cfg(windows)]
    fn write_tool(dir: &Path) {
        std::fs::write(dir.join(TOOL), "@echo tool\r\n").unwrap();
    }

    /// Runs a child with a timeout from the process of a test that holds
    /// the fifo in `dir` open and exits the process without ending the run
    /// once the child is ready.
    #[cfg(target_os = "linux")]
    fn exit_while_running_child(dir: PathBuf) -> ! {
        struct ExitOnOutput;

        impl Interceptor<ProcessCommand> for ExitOnOutput {
            fn chunk(&self, _chunk: crate::interceptor::Chunk<ProcessCommand>) -> Result {
                std::process::exit(0);
            }
        }

        let process = process(dir);
        process.add_interceptor(ExitOnOutput);
        let _ = shell("exec 3> fifo; echo ready; exec sleep 10")
            .stream()
            .timeout(Duration::from_secs(10))
            .run_blocking(&process);
        std::process::exit(1)
    }

    mod std_executor {
        use super::*;

        mod executor {
            use super::*;

            mod execute {
                use super::*;

                #[test]
                fn execute_should_run_attempts_of_retried_command() {
                    // GIVEN a command failing on its first attempt only
                    let dir = TestDir::new();
                    let (process, recorder) = observed();
                    process.set_current_dir(dir.path().to_path_buf());

                    // WHEN running it with retries
                    let output = shell(FAIL_ONCE)
                        .interceptors(retried(Retry::new(2, Duration::from_millis(1)), &recorder))
                        .run_blocking(&process)
                        .unwrap();

                    // THEN the second attempt succeeds
                    assert_eq!(output.status, Some(0));
                    assert_eq!(
                        recorder.events()[1..],
                        [
                            "finished failed Some(1)".to_string(),
                            format!("started in {}", dir.path().display()),
                            "finished ok".to_string()
                        ]
                    );
                }
            }

            mod sleep {
                use super::*;

                #[test]
                fn sleep_should_block_thread_for_delay() {
                    let start = Instant::now();

                    let _ = shell("exit 1")
                        .interceptors(retried(
                            Retry::new(1, Duration::from_millis(50)),
                            &Recorder::default(),
                        ))
                        .run_blocking(&process(std::env::temp_dir()));

                    assert!(start.elapsed() >= Duration::from_millis(50));
                }
            }
        }
    }

    #[cfg(feature = "tokio")]
    mod tokio_executor {
        use super::*;

        mod executor {
            use super::*;

            mod execute {
                use super::*;

                #[test]
                fn execute_should_run_attempts_of_retried_command() {
                    let dir = TestDir::new();
                    let (process, recorder) = observed();
                    process.set_current_dir(dir.path().to_path_buf());

                    let output = block_on(
                        shell(FAIL_ONCE)
                            .interceptors(retried(
                                Retry::new(2, Duration::from_millis(1)),
                                &recorder,
                            ))
                            .run_async(&process),
                    )
                    .unwrap();

                    assert_eq!(output.status, Some(0));
                    assert_eq!(recorder.events().len(), 4);
                }
            }

            mod sleep {
                use super::*;

                #[test]
                fn sleep_should_wait_for_delay() {
                    let start = Instant::now();

                    let _ = block_on(
                        shell("exit 1")
                            .interceptors(retried(
                                Retry::new(1, Duration::from_millis(50)),
                                &Recorder::default(),
                            ))
                            .run_async(&process(std::env::temp_dir())),
                    );

                    assert!(start.elapsed() >= Duration::from_millis(50));
                }

                #[test]
                fn sleep_cancelled_should_tell_interceptors() {
                    // GIVEN a recorder outside a retry waiting long after a failure
                    let (process, recorder) = observed();
                    let mut interceptors = Interceptors::new();
                    interceptors.push(recorder.clone());
                    interceptors.push(Retry::new(1, Duration::from_secs(3600)));

                    // WHEN the run is cancelled while it waits
                    let result = block_on(async {
                        tokio::time::timeout(
                            Duration::from_millis(200),
                            shell("exit 1")
                                .interceptors(interceptors)
                                .run_async(&process),
                        )
                        .await
                    });

                    // THEN the interceptors are told
                    assert!(result.is_err());
                    assert_eq!(
                        *recorder.events().last().unwrap(),
                        format!("cancelled {SHELL}")
                    );
                }
            }
        }
    }

    mod process_impl {
        use super::*;

        mod process {
            use super::*;

            mod run_blocking {
                use super::*;

                #[test]
                fn run_blocking_should_capture_output() {
                    crate::process::tests::process::run_blocking::run_blocking_should_capture_output(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& echo err 1>&2"),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn run_blocking_killed_by_signal_should_report_signal() {
                    crate::process::tests::process::run_blocking::run_blocking_killed_by_signal_should_report_signal(
                        &process(std::env::temp_dir()),
                        &shell("kill -9 $$"),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn run_blocking_large_input_should_be_written_whole() {
                    let process = process(std::env::temp_dir());
                    let input = vec![b'x'; 1 << 20];

                    let output = process
                        .run_blocking(Run::from(&shell("wc -c")).input(input))
                        .unwrap();

                    assert_eq!(output.stdout_str().unwrap().trim(), "1048576");
                }

                #[test]
                fn run_blocking_streaming_should_capture_output() {
                    let process = process(std::env::temp_dir());
                    let options = CommandOptions {
                        display_lines: 10,
                        clear_env: false,
                        silent: false,
                        input: None,
                        interactive: false,
                        stream: true,
                        timeout: None,
                    };

                    let output = process
                        .run_blocking(with_options(
                            &shell("echo out&& echo err 1>&2"),
                            options.clone(),
                        ))
                        .unwrap();

                    assert_eq!(output.stdout_str().unwrap().trim(), "out");
                    assert_eq!(output.stderr_str().unwrap().trim(), "err");
                }

                #[test]
                fn run_blocking_streaming_should_pass_output_to_interceptors() {
                    crate::process::tests::process::run_blocking::run_blocking_streaming_should_pass_output_to_interceptors(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& echo err 1>&2"),
                    );
                }

                #[test]
                fn run_blocking_silent_should_neither_capture_nor_stream_output() {
                    crate::process::tests::process::run_blocking::run_blocking_silent_should_neither_capture_nor_stream_output(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& echo err 1>&2"),
                    );
                }

                #[test]
                fn run_blocking_should_pass_run_and_result_to_interceptors() {
                    crate::process::tests::process::run_blocking::run_blocking_should_pass_run_and_result_to_interceptors(
                        &process(std::env::temp_dir()),
                        &shell("exit 0"),
                        &shell("exit 3"),
                        &std::env::temp_dir(),
                    );
                }

                #[test]
                fn run_blocking_should_run_in_current_dir() {
                    let dir = TestDir::new();
                    let process = process(dir.path().to_path_buf());

                    let output = process.run_blocking(&shell(PRINT_DIR)).unwrap();

                    assert_eq!(
                        std::fs::canonicalize(output.stdout_str().unwrap().trim()).unwrap(),
                        std::fs::canonicalize(dir.path()).unwrap()
                    );
                }

                #[test]
                fn run_blocking_relative_dir_should_resolve_against_current_dir() {
                    // GIVEN a sub directory of the current directory
                    let dir = TestDir::new();
                    std::fs::create_dir(dir.path().join("sub")).unwrap();
                    let process = process(dir.path().to_path_buf());

                    // WHEN running a command in it by a relative path
                    let output = process
                        .run_blocking(shell(PRINT_DIR).current_dir("sub"))
                        .unwrap();

                    // THEN the command runs in the sub directory
                    assert_eq!(
                        std::fs::canonicalize(output.stdout_str().unwrap().trim()).unwrap(),
                        std::fs::canonicalize(dir.path().join("sub")).unwrap()
                    );
                }

                #[test]
                fn run_blocking_relative_program_should_resolve_against_dir() {
                    // GIVEN a program in a sub directory of the current directory
                    let dir = TestDir::new();
                    std::fs::create_dir(dir.path().join("sub")).unwrap();
                    write_tool(&dir.path().join("sub"));
                    let process = process(dir.path().to_path_buf());

                    // WHEN running it by a path relative to the sub directory
                    // it runs in
                    let output = process
                        .run_blocking(Command::new(Path::new(".").join(TOOL)).current_dir("sub"))
                        .unwrap();

                    // THEN it is found there
                    assert_eq!(output.stdout_str().unwrap().trim(), "tool");
                }

                #[test]
                fn run_blocking_should_pass_vars_of_platform() {
                    let process = ProcessImpl::new(std::env::temp_dir());

                    let output = process.run_blocking(&print_var("PATH")).unwrap();

                    assert_ne!(output.stdout_str().unwrap().trim(), "unset");
                }

                #[test]
                fn run_blocking_unchanged_vars_of_process_should_inherit_them() {
                    // GIVEN a platform with the unchanged variables of the process
                    let process = ProcessImpl::with_env(EnvImpl::new_from_std());

                    // WHEN running commands with and without own variables
                    let outputs = [
                        process.run_blocking(&print_var(INHERITED_VAR)).unwrap(),
                        process
                            .run_blocking(print_var(INHERITED_VAR).env("AGPLATFORM_VAR", "1"))
                            .unwrap(),
                    ];

                    // THEN they get the variables of the process
                    let expected = std::env::var(INHERITED_VAR).unwrap();
                    for output in outputs {
                        assert_eq!(output.stdout_str().unwrap().trim(), expected);
                    }
                }

                #[test]
                fn run_blocking_should_not_pass_removed_inherited_var() {
                    let process = process(std::env::temp_dir());
                    process.remove_var(INHERITED_VAR);

                    let output = process.run_blocking(&print_var(INHERITED_VAR)).unwrap();

                    assert_eq!(output.stdout_str().unwrap().trim(), "unset");
                }

                #[test]
                fn run_blocking_should_pass_vars() {
                    let process = process(std::env::temp_dir());
                    process.set_var("AGPLATFORM_VAR".to_string(), "value".to_string());

                    let output = process.run_blocking(&print_var("AGPLATFORM_VAR")).unwrap();

                    assert_eq!(output.stdout_str().unwrap().trim(), "value");
                }

                #[test]
                fn run_blocking_command_vars_should_override_vars() {
                    // GIVEN configured variables
                    let process = process(std::env::temp_dir());
                    process.set_var("AGPLATFORM_SET".to_string(), "1".to_string());
                    process.set_var("AGPLATFORM_REMOVED".to_string(), "1".to_string());

                    // WHEN commands set or remove them
                    let set = process
                        .run_blocking(print_var("AGPLATFORM_SET").env("AGPLATFORM_SET", "2"))
                        .unwrap();
                    let removed = process
                        .run_blocking(
                            print_var("AGPLATFORM_REMOVED").env_remove("AGPLATFORM_REMOVED"),
                        )
                        .unwrap();

                    // THEN the commands win
                    assert_eq!(set.stdout_str().unwrap().trim(), "2");
                    assert_eq!(removed.stdout_str().unwrap().trim(), "unset");
                }

                #[test]
                fn run_blocking_clear_env_should_pass_only_command_vars() {
                    // GIVEN a configured variable
                    let process = process(std::env::temp_dir());
                    process.set_var("AGPLATFORM_CONFIGURED".to_string(), "1".to_string());
                    let options = CommandOptions {
                        display_lines: 10,
                        clear_env: true,
                        ..Default::default()
                    };

                    // WHEN running commands without the configured variables
                    let configured = process
                        .run_blocking(with_options(
                            &print_var("AGPLATFORM_CONFIGURED"),
                            options.clone(),
                        ))
                        .unwrap();
                    let own = process
                        .run_blocking(with_options(
                            print_var("AGPLATFORM_OWN").env("AGPLATFORM_OWN", "2"),
                            options.clone(),
                        ))
                        .unwrap();

                    // THEN only the variables of the commands are passed
                    assert_eq!(configured.stdout_str().unwrap().trim(), "unset");
                    assert_eq!(own.stdout_str().unwrap().trim(), "2");
                }

                #[cfg(unix)]
                #[test]
                fn run_blocking_changed_path_should_find_program_in_it() {
                    // GIVEN a program found only in a directory
                    let dir = TestDir::new();
                    write_tool(dir.path());
                    let path = dir.path().display().to_string();
                    let configured = process(std::env::temp_dir());
                    configured.set_var("PATH".to_string(), path.clone());
                    let unconfigured = process(std::env::temp_dir());
                    let mut own = Command::new(TOOL);
                    own.env("PATH", &path);

                    // WHEN the platform or the command sets the PATH to it,
                    // with and without the configured variables
                    let outputs = [
                        configured.run_blocking(&Command::new(TOOL)),
                        configured.run_blocking(Command::new(TOOL).clear_env()),
                        unconfigured.run_blocking(&own),
                        unconfigured.run_blocking(own.clear_env()),
                    ];

                    // THEN the program is found
                    for (index, output) in outputs.into_iter().enumerate() {
                        assert_eq!(
                            output.unwrap().stdout_str().unwrap().trim(),
                            "tool",
                            "case {index}"
                        );
                    }
                }

                #[cfg(unix)]
                #[test]
                fn run_blocking_without_path_should_find_program_in_default_path() {
                    // GIVEN a platform without a PATH and one whose command
                    // removes it
                    let without =
                        ProcessImpl::with_env(EnvImpl::new(Vec::new(), std::env::temp_dir()));
                    let process = process(std::env::temp_dir());

                    // WHEN running a program by its bare name
                    let outputs = [
                        without.run_blocking(Command::new("sh").args(["-c", "echo found"])),
                        process.run_blocking(
                            Command::new("sh")
                                .args(["-c", "echo found"])
                                .env_remove("PATH"),
                        ),
                    ];

                    // THEN std finds it in the default path of the OS
                    for (index, output) in outputs.into_iter().enumerate() {
                        assert_eq!(
                            output.unwrap().stdout_str().unwrap().trim(),
                            "found",
                            "case {index}"
                        );
                    }
                }

                #[cfg(unix)]
                #[test]
                fn run_blocking_missing_program_in_changed_path_should_return_not_found() {
                    let dir = TestDir::new();
                    let process = process(std::env::temp_dir());
                    process.set_var("PATH".to_string(), dir.path().display().to_string());

                    let error = process
                        .run_blocking(&Command::new("agplatform_missing_program"))
                        .unwrap_err();

                    assert_eq!(
                        error.description(),
                        "'agplatform_missing_program': No such file or directory (os error 2)"
                    );
                    assert_eq!(
                        error
                            .source_as::<std::io::Error>()
                            .map(std::io::Error::kind),
                        Some(std::io::ErrorKind::NotFound)
                    );
                }

                #[test]
                fn run_blocking_input_should_write_input_to_stdin() {
                    let process = process(std::env::temp_dir());
                    let options = CommandOptions {
                        input: Some(b"b\na\n".to_vec()),
                        ..Default::default()
                    };

                    let output = process
                        .run_blocking(with_options(&echo_stdin(), options.clone()))
                        .unwrap();

                    assert_eq!(
                        output.stdout_str().unwrap().lines().collect::<Vec<_>>(),
                        ["b", "a"]
                    );
                }

                #[test]
                fn run_blocking_input_larger_than_pipe_should_not_deadlock() {
                    // GIVEN input exceeding the pipe buffers
                    let process = process(std::env::temp_dir());
                    let options = CommandOptions {
                        input: Some("a\n".repeat(500_000).into_bytes()),
                        ..Default::default()
                    };

                    // WHEN the command echoes it back while reading
                    let output = process
                        .run_blocking(with_options(&echo_stdin(), options.clone()))
                        .unwrap();

                    // THEN it is read and output whole
                    assert_eq!(output.stdout_str().unwrap().lines().count(), 500_000);
                }

                #[test]
                fn run_blocking_input_not_read_should_succeed() {
                    let process = process(std::env::temp_dir());
                    let options = CommandOptions {
                        input: Some("a\n".repeat(500_000).into_bytes()),
                        ..Default::default()
                    };

                    let output = process
                        .run_blocking(with_options(&shell("exit 0"), options.clone()))
                        .unwrap();

                    assert_eq!(output.status, Some(0));
                }

                #[test]
                fn run_blocking_interactive_should_not_capture_output() {
                    crate::process::tests::process::run_blocking::run_blocking_interactive_should_not_capture_output(
                        &process(std::env::temp_dir()),
                        &shell("exit 3"),
                    );
                }

                #[test]
                fn run_blocking_invalid_options_should_return_error() {
                    crate::process::tests::process::run_blocking::run_blocking_invalid_options_should_return_error(
                        &process(std::env::temp_dir()),
                        &shell("exit 0"),
                    );
                }

                #[test]
                fn run_blocking_should_leave_command_unchanged() {
                    let process = process(std::env::temp_dir());
                    let command = shell("echo out");

                    process.run_blocking(&command).unwrap();

                    assert_eq!(command.get_current_dir(), None);
                    assert_eq!(command.get_envs().count(), 0);
                }

                #[test]
                fn run_blocking_non_zero_exit_should_return_error_with_output() {
                    crate::process::tests::process::run_blocking::run_blocking_non_zero_exit_should_return_error_with_output(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& exit 3"),
                    );
                }

                #[test]
                fn run_blocking_missing_program_should_return_error_with_io_source() {
                    crate::process::tests::process::run_blocking::run_blocking_missing_program_should_return_error_with_io_source(
                        &process(std::env::temp_dir()),
                        &Command::new("agplatform_missing_program"),
                    );
                }

                #[test]
                fn run_blocking_within_timeout_should_succeed() {
                    crate::process::tests::process::run_blocking::run_blocking_within_timeout_should_succeed(
                        &process(std::env::temp_dir()),
                        &shell("echo out"),
                    );
                }

                #[test]
                fn run_blocking_maximum_timeout_should_not_limit_run() {
                    crate::process::tests::process::run_blocking::run_blocking_maximum_timeout_should_not_limit_run(
                        &process(std::env::temp_dir()),
                        &shell("echo out"),
                    );
                }

                #[test]
                fn run_blocking_exited_child_should_not_wait_for_descendants_holding_output() {
                    // GIVEN a child that exits leaving a sleeping descendant
                    // holding its output open
                    let process = process(std::env::temp_dir());
                    let command = shell(BACKGROUND_SLEEPER);

                    for timeout in [None, Some(Duration::from_secs(10))] {
                        let options = CommandOptions {
                            timeout,
                            ..Default::default()
                        };

                        // WHEN running it
                        let output = process
                            .run_blocking(with_options(&command, options.clone()))
                            .unwrap();
                        kill_sleeper(output.stdout_str().unwrap());

                        // THEN the run ends with the child, with the output
                        // before its exit
                        assert_eq!(output.status, Some(0), "timeout: {timeout:?}");
                        assert!(
                            output.stdout_str().unwrap().trim().parse::<u32>().is_ok(),
                            "timeout: {timeout:?}"
                        );
                        assert!(
                            output.duration < EXITED_CHILD_WAIT_MAX,
                            "timeout: {timeout:?}"
                        );
                    }
                }

                #[cfg(target_os = "linux")]
                #[test]
                fn run_blocking_timeout_exited_parent_should_kill_child() {
                    const PARENT_DIR: &str = "AGPLATFORM_TEST_PARENT_DIR";

                    if let Some(dir) = std::env::var_os(PARENT_DIR) {
                        exit_while_running_child(PathBuf::from(dir));
                    }

                    // GIVEN a parent process running a child with a timeout,
                    // which holds a fifo open
                    let dir = TestDir::new();
                    assert!(
                        Command::new("mkfifo")
                            .arg(dir.path().join("fifo"))
                            .status()
                            .unwrap()
                            .success()
                    );
                    let test = format!(
                        "{}::run_blocking_timeout_exited_parent_should_kill_child",
                        module_path!().split_once("::").unwrap().1
                    );
                    let mut parent = Command::new(std::env::current_exe().unwrap())
                        .args([test.as_str(), "--exact"])
                        .env(PARENT_DIR, dir.path())
                        .stdout(Stdio::null())
                        .spawn()
                        .unwrap();
                    let mut fifo = std::fs::File::open(dir.path().join("fifo")).unwrap();
                    let start = Instant::now();

                    // WHEN the parent exits without ending the run
                    assert!(parent.wait().unwrap().success());

                    // THEN the child is killed, closing the fifo
                    std::io::Read::read_to_end(&mut fifo, &mut Vec::new()).unwrap();
                    assert!(start.elapsed() < Duration::from_secs(5));
                }

                #[test]
                fn run_blocking_exceeding_timeout_should_kill_process_tree() {
                    // GIVEN a shell whose sleeping child holds its output open
                    let process = process(std::env::temp_dir());
                    let options = CommandOptions {
                        display_lines: 10,
                        clear_env: false,
                        silent: false,
                        input: None,
                        interactive: false,
                        stream: false,
                        timeout: Some(Duration::from_millis(200)),
                    };

                    // WHEN it exceeds the timeout
                    let error = process
                        .run_blocking(with_options(&shell(SLEEP_THEN_ECHO), options.clone()))
                        .unwrap_err();

                    // THEN both are killed so the output ends right away
                    let output = error.command_output().unwrap();
                    assert!(output.timed_out);
                    assert_eq!(output.status, None);
                    assert!(output.stdout.is_empty());
                    assert!(output.duration < Duration::from_secs(5));
                }

                #[test]
                fn run_blocking_zero_timeout_should_time_out() {
                    crate::process::tests::process::run_blocking::run_blocking_zero_timeout_should_time_out(
                        &process(std::env::temp_dir()),
                        &shell("echo out"),
                    );
                }

                #[test]
                fn run_blocking_exceeding_timeout_should_time_out_keeping_output() {
                    crate::process::tests::process::run_blocking::run_blocking_exceeding_timeout_should_time_out_keeping_output(
                        &process(std::env::temp_dir()),
                        &shell(ECHO_THEN_SLEEP),
                        Duration::from_millis(500),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn run_blocking_exceeding_timeout_after_closing_output_should_time_out() {
                    let process = process(std::env::temp_dir());
                    let start = Instant::now();

                    let error = process
                        .run_blocking(
                            shell("exec >&- 2>&-; sleep 10").timeout(Duration::from_millis(300)),
                        )
                        .unwrap_err();

                    assert!(error.command_output().unwrap().timed_out);
                    assert!(start.elapsed() < Duration::from_secs(5));
                }

                #[cfg(unix)]
                #[test]
                fn run_blocking_input_not_read_exceeding_timeout_should_time_out() {
                    // GIVEN a command not reading input larger than the pipe
                    let process = process(std::env::temp_dir());
                    let start = Instant::now();

                    // WHEN it exceeds the timeout
                    let error = process
                        .run_blocking(
                            shell("sleep 10")
                                .input(vec![b'x'; 1 << 20])
                                .timeout(Duration::from_millis(300)),
                        )
                        .unwrap_err();

                    // THEN it is killed without waiting for the input to be read
                    assert!(error.command_output().unwrap().timed_out);
                    assert!(start.elapsed() < Duration::from_secs(5));
                }

                #[cfg(unix)]
                #[test]
                fn run_blocking_large_output_on_both_streams_should_be_captured_whole() {
                    let process = process(std::env::temp_dir());

                    let output = process
                        .run_blocking(&shell(LARGE_OUTPUT_ON_BOTH_STREAMS))
                        .unwrap();

                    assert_eq!(output.stdout.len(), 1_000_000);
                    assert_eq!(output.stderr.len(), 1_000_000);
                }

                #[cfg(target_os = "linux")]
                #[test]
                fn run_blocking_bare_program_should_get_its_name_as_first_arg() {
                    // WHY: std passes the program as given as argv[0], which
                    // e.g. multi-call binaries and login shells read.
                    let process = process(std::env::temp_dir());

                    let output = process
                        .run_blocking(&shell("cat /proc/$$/cmdline; :"))
                        .unwrap();

                    assert_eq!(
                        String::from_utf8_lossy(&output.stdout),
                        "sh\0-c\0cat /proc/$$/cmdline; :\0"
                    );
                }

                #[test]
                fn run_blocking_env_clear_of_command_should_be_ignored() {
                    let process = process(std::env::temp_dir());
                    process.set_var("AGPLATFORM_VAR".to_string(), "value".to_string());

                    let output = process
                        .run_blocking(print_var("AGPLATFORM_VAR").env_clear())
                        .unwrap();

                    assert_eq!(output.stdout_str().unwrap().trim(), "value");
                }

                #[cfg(unix)]
                #[test]
                fn run_blocking_not_executable_program_should_return_permission_denied() {
                    let dir = TestDir::new();
                    std::fs::write(dir.path().join(TOOL), "#!/bin/sh\necho tool\n").unwrap();
                    let process = process(dir.path().to_path_buf());

                    let error = process
                        .run_blocking(&Command::new(Path::new(".").join(TOOL)))
                        .unwrap_err();

                    assert_eq!(
                        error
                            .source_as::<std::io::Error>()
                            .map(std::io::Error::kind),
                        Some(std::io::ErrorKind::PermissionDenied)
                    );
                }
            }

            #[cfg(feature = "tokio")]
            mod run_async {
                use super::*;

                #[test]
                fn run_async_should_capture_output() {
                    crate::process::tests::process::run_async::run_async_should_capture_output(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& echo err 1>&2"),
                    );
                }

                #[test]
                fn run_async_streaming_should_pass_output_to_interceptors() {
                    crate::process::tests::process::run_async::run_async_streaming_should_pass_output_to_interceptors(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& echo err 1>&2"),
                    );
                }

                #[test]
                fn run_async_silent_should_neither_capture_nor_stream_output() {
                    crate::process::tests::process::run_async::run_async_silent_should_neither_capture_nor_stream_output(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& echo err 1>&2"),
                    );
                }

                #[test]
                fn run_async_input_should_write_input_to_stdin() {
                    let process = process(std::env::temp_dir());
                    let options = CommandOptions {
                        input: Some(b"b\na\n".to_vec()),
                        ..Default::default()
                    };

                    let output =
                        block_on(process.run_async(with_options(&echo_stdin(), options.clone())))
                            .unwrap();

                    assert_eq!(
                        output.stdout_str().unwrap().lines().collect::<Vec<_>>(),
                        ["b", "a"]
                    );
                }

                #[test]
                fn run_async_input_larger_than_pipe_should_not_deadlock() {
                    // GIVEN input exceeding the pipe buffers
                    let process = process(std::env::temp_dir());
                    let options = CommandOptions {
                        input: Some("a\n".repeat(500_000).into_bytes()),
                        ..Default::default()
                    };

                    // WHEN the command echoes it back while reading
                    let output =
                        block_on(process.run_async(with_options(&echo_stdin(), options.clone())))
                            .unwrap();

                    // THEN it is read and output whole
                    assert_eq!(output.stdout_str().unwrap().lines().count(), 500_000);
                }

                #[test]
                fn run_async_input_not_read_should_succeed() {
                    let process = process(std::env::temp_dir());
                    let options = CommandOptions {
                        input: Some("a\n".repeat(500_000).into_bytes()),
                        ..Default::default()
                    };

                    let output = block_on(
                        process.run_async(with_options(&shell("exit 0"), options.clone())),
                    )
                    .unwrap();

                    assert_eq!(output.status, Some(0));
                }

                #[test]
                fn run_async_interactive_should_not_capture_output() {
                    crate::process::tests::process::run_async::run_async_interactive_should_not_capture_output(
                        &process(std::env::temp_dir()),
                        &shell("exit 3"),
                    );
                }

                #[test]
                fn run_async_invalid_options_should_return_error() {
                    crate::process::tests::process::run_async::run_async_invalid_options_should_return_error(
                        &process(std::env::temp_dir()),
                        &shell("exit 0"),
                    );
                }

                #[test]
                fn run_async_missing_program_should_return_error_with_io_source() {
                    crate::process::tests::process::run_async::run_async_missing_program_should_return_error_with_io_source(
                        &process(std::env::temp_dir()),
                        &Command::new("agplatform_missing_program"),
                    );
                }

                #[test]
                fn run_async_within_timeout_should_succeed() {
                    crate::process::tests::process::run_async::run_async_within_timeout_should_succeed(
                        &process(std::env::temp_dir()),
                        &shell("echo out"),
                    );
                }

                #[test]
                fn run_async_maximum_timeout_should_not_limit_run() {
                    crate::process::tests::process::run_async::run_async_maximum_timeout_should_not_limit_run(
                        &process(std::env::temp_dir()),
                        &shell("echo out"),
                    );
                }

                #[test]
                fn run_async_exited_child_should_not_wait_for_descendants_holding_output() {
                    // GIVEN a child that exits leaving a sleeping descendant
                    // holding its output open
                    let process = process(std::env::temp_dir());
                    let command = shell(BACKGROUND_SLEEPER);

                    for timeout in [None, Some(Duration::from_secs(10))] {
                        let options = CommandOptions {
                            timeout,
                            ..Default::default()
                        };

                        // WHEN running it
                        let output =
                            block_on(process.run_async(with_options(&command, options.clone())))
                                .unwrap();
                        kill_sleeper(output.stdout_str().unwrap());

                        // THEN the run ends with the child, with the output
                        // before its exit
                        assert_eq!(output.status, Some(0), "timeout: {timeout:?}");
                        assert!(
                            output.stdout_str().unwrap().trim().parse::<u32>().is_ok(),
                            "timeout: {timeout:?}"
                        );
                        assert!(
                            output.duration < EXITED_CHILD_WAIT_MAX,
                            "timeout: {timeout:?}"
                        );
                    }
                }

                #[test]
                fn run_async_exceeding_timeout_should_kill_process_tree_keeping_output() {
                    // GIVEN a shell printing before its sleeping child holds its
                    // output open
                    let process = process(std::env::temp_dir());
                    let options = CommandOptions {
                        timeout: Some(Duration::from_millis(500)),
                        ..Default::default()
                    };

                    // WHEN it exceeds the timeout
                    let error = block_on(
                        process.run_async(with_options(&shell(ECHO_THEN_SLEEP), options.clone())),
                    )
                    .unwrap_err();

                    // THEN both are killed and the output so far is kept
                    let output = error.command_output().unwrap();
                    assert!(output.timed_out);
                    assert_eq!(output.status, None);
                    assert_eq!(output.stdout_str().unwrap().trim(), "out");
                    assert!(output.duration < Duration::from_secs(5));
                }

                #[test]
                fn run_async_cancelled_should_kill_process_tree() {
                    // GIVEN a child with a descendant, which would create a
                    // marker later
                    let dir = TestDir::new();
                    let process = process(dir.path().to_path_buf());
                    let command = shell(MARK_LATER_THEN_SLEEP);

                    // WHEN the run is cancelled by dropping it
                    let result = block_on(async {
                        tokio::time::timeout(
                            Duration::from_millis(200),
                            process.run_async(&command),
                        )
                        .await
                    });
                    std::thread::sleep(AFTER_MARKER);

                    // THEN the descendant was killed before creating the marker
                    assert!(result.is_err());
                    assert!(!dir.path().join("marker").exists());
                }

                #[test]
                fn run_async_non_zero_exit_should_return_error_with_output() {
                    crate::process::tests::process::run_async::run_async_non_zero_exit_should_return_error_with_output(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& exit 3"),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn run_async_killed_by_signal_should_report_signal() {
                    crate::process::tests::process::run_async::run_async_killed_by_signal_should_report_signal(
                        &process(std::env::temp_dir()),
                        &shell("kill -9 $$"),
                    );
                }

                #[test]
                fn run_async_zero_timeout_should_time_out() {
                    crate::process::tests::process::run_async::run_async_zero_timeout_should_time_out(
                        &process(std::env::temp_dir()),
                        &shell("echo out"),
                    );
                }

                #[test]
                fn run_async_exceeding_timeout_should_time_out_keeping_output() {
                    crate::process::tests::process::run_async::run_async_exceeding_timeout_should_time_out_keeping_output(
                        &process(std::env::temp_dir()),
                        &shell(ECHO_THEN_SLEEP),
                        Duration::from_millis(500),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn run_async_exceeding_timeout_after_closing_output_should_time_out() {
                    let process = process(std::env::temp_dir());
                    let start = Instant::now();

                    let error = block_on(process.run_async(
                        shell("exec >&- 2>&-; sleep 10").timeout(Duration::from_millis(300)),
                    ))
                    .unwrap_err();

                    assert!(error.command_output().unwrap().timed_out);
                    assert!(start.elapsed() < Duration::from_secs(5));
                }

                #[cfg(unix)]
                #[test]
                fn run_async_input_not_read_exceeding_timeout_should_time_out() {
                    // GIVEN a command not reading input larger than the pipe
                    let process = process(std::env::temp_dir());
                    let start = Instant::now();

                    // WHEN it exceeds the timeout
                    let error = block_on(
                        process.run_async(
                            shell("sleep 10")
                                .input(vec![b'x'; 1 << 20])
                                .timeout(Duration::from_millis(300)),
                        ),
                    )
                    .unwrap_err();

                    // THEN it is killed without waiting for the input to be read
                    assert!(error.command_output().unwrap().timed_out);
                    assert!(start.elapsed() < Duration::from_secs(5));
                }

                #[cfg(target_os = "linux")]
                #[test]
                fn run_async_bare_program_should_get_its_name_as_first_arg() {
                    // WHY: std passes the program as given as argv[0], which
                    // e.g. multi-call binaries and login shells read.
                    let process = process(std::env::temp_dir());

                    let output =
                        block_on(process.run_async(&shell("cat /proc/$$/cmdline; :"))).unwrap();

                    assert_eq!(
                        String::from_utf8_lossy(&output.stdout),
                        "sh\0-c\0cat /proc/$$/cmdline; :\0"
                    );
                }

                #[cfg(unix)]
                #[test]
                fn run_async_large_output_on_both_streams_should_be_captured_whole() {
                    let process = process(std::env::temp_dir());

                    let output =
                        block_on(process.run_async(&shell(LARGE_OUTPUT_ON_BOTH_STREAMS))).unwrap();

                    assert_eq!(output.stdout.len(), 1_000_000);
                    assert_eq!(output.stderr.len(), 1_000_000);
                }

                #[test]
                fn run_async_cancelled_should_tell_interceptors() {
                    crate::process::tests::process::run_async::run_async_cancelled_should_tell_interceptors(
                        &process(std::env::temp_dir()),
                        &shell(SLEEP_THEN_ECHO),
                    );
                }
            }

            mod run_blocking_with_sink {
                use super::*;

                #[test]
                fn run_blocking_with_sink_should_pass_output_to_sink_instead_of_capturing() {
                    crate::process::tests::process::run_blocking_with_sink::run_blocking_with_sink_should_pass_output_to_sink_instead_of_capturing(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& echo err 1>&2"),
                    );
                }

                #[test]
                fn run_blocking_with_sink_should_pass_output_through_interceptors_to_sink() {
                    crate::process::tests::process::run_blocking_with_sink::run_blocking_with_sink_should_pass_output_through_interceptors_to_sink(
                        &process(std::env::temp_dir()),
                        &shell("echo out"),
                    );
                }

                #[test]
                fn run_blocking_with_sink_failing_sink_should_kill_child_and_return_sink_error() {
                    // GIVEN a command that writes and then runs for long
                    let process = process(std::env::temp_dir());
                    let command = shell(ECHO_THEN_SLEEP);
                    let start = Instant::now();

                    // WHEN its sink fails on the first chunk
                    let error = process
                        .run_blocking_with_sink(&command, &mut |_: OutputStream, _: &[u8]| {
                            Err(crate::Error::process("full"))
                        })
                        .unwrap_err();

                    // THEN the run ends at once with the error of the sink
                    assert_eq!(error.description(), "full");
                    assert!(start.elapsed() < Duration::from_secs(5));
                }

                #[cfg(unix)]
                #[test]
                fn run_blocking_with_sink_failing_sink_with_timeout_should_kill_process_tree() {
                    // GIVEN a child with a descendant, which would create a
                    // marker later, and a timeout too long to limit the run
                    let dir = TestDir::new();
                    let process = process(dir.path().to_path_buf());
                    let command = shell(MARK_LATER_THEN_ECHO_THEN_SLEEP).timeout(Duration::MAX);

                    // WHEN its sink fails on the first chunk
                    let error = process
                        .run_blocking_with_sink(command, &mut |_: OutputStream, _: &[u8]| {
                            Err(crate::Error::process("full"))
                        })
                        .unwrap_err();
                    std::thread::sleep(Duration::from_millis(1000));

                    // THEN the descendant was killed before creating the marker
                    assert_eq!(error.description(), "full");
                    assert!(!dir.path().join("marker").exists());
                }

                #[cfg(unix)]
                #[test]
                fn run_blocking_with_sink_failing_sink_without_timeout_should_leave_descendants_running()
                 {
                    // GIVEN a child with a descendant, which creates a marker
                    // later
                    let dir = TestDir::new();
                    let process = process(dir.path().to_path_buf());
                    let command = shell(MARK_LATER_THEN_ECHO_THEN_SLEEP);

                    // WHEN its sink fails on the first chunk
                    let error = process
                        .run_blocking_with_sink(&command, &mut |_: OutputStream, _: &[u8]| {
                            Err(crate::Error::process("full"))
                        })
                        .unwrap_err();
                    std::thread::sleep(Duration::from_millis(1000));

                    // THEN only the child was killed as it leads no process
                    // group, so the descendant created the marker
                    assert_eq!(error.description(), "full");
                    assert!(dir.path().join("marker").exists());
                }

                #[test]
                fn run_blocking_with_sink_failed_should_return_error_without_output() {
                    crate::process::tests::process::run_blocking_with_sink::run_blocking_with_sink_failed_should_return_error_without_output(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& echo err 1>&2&& exit 3"),
                    );
                }

                #[test]
                fn run_blocking_with_sink_failing_sink_should_return_its_error() {
                    crate::process::tests::process::run_blocking_with_sink::run_blocking_with_sink_failing_sink_should_return_its_error(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& echo err 1>&2"),
                    );
                }

                #[test]
                fn run_blocking_with_sink_silent_should_fail_without_running() {
                    crate::process::tests::process::run_blocking_with_sink::run_blocking_with_sink_silent_should_fail_without_running(
                        &process(std::env::temp_dir()),
                        &shell("echo out"),
                    );
                }
            }

            #[cfg(feature = "tokio")]
            mod run_async_with_sink {
                use super::*;

                #[test]
                fn run_async_with_sink_should_pass_output_to_sink_instead_of_capturing() {
                    crate::process::tests::process::run_async_with_sink::run_async_with_sink_should_pass_output_to_sink_instead_of_capturing(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& echo err 1>&2"),
                    );
                }

                #[test]
                fn run_async_with_sink_failing_sink_should_kill_child_and_return_sink_error() {
                    let process = process(std::env::temp_dir());
                    let command = shell(ECHO_THEN_SLEEP);
                    let start = Instant::now();

                    let error = block_on(
                        process.run_async_with_sink(&command, &mut |_: OutputStream, _: &[u8]| {
                            Err(crate::Error::process("full"))
                        }),
                    )
                    .unwrap_err();

                    assert_eq!(error.description(), "full");
                    assert!(start.elapsed() < Duration::from_secs(5));
                }

                #[cfg(unix)]
                #[test]
                fn run_async_with_sink_failing_sink_should_kill_process_tree() {
                    // GIVEN a child with a descendant, which would create a
                    // marker later
                    let dir = TestDir::new();
                    let process = process(dir.path().to_path_buf());
                    let command = shell(MARK_LATER_THEN_ECHO_THEN_SLEEP);

                    // WHEN its sink fails on the first chunk
                    let error = block_on(
                        process.run_async_with_sink(&command, &mut |_: OutputStream, _: &[u8]| {
                            Err(crate::Error::process("full"))
                        }),
                    )
                    .unwrap_err();
                    std::thread::sleep(Duration::from_millis(1000));

                    // THEN the descendant was killed before creating the marker
                    assert_eq!(error.description(), "full");
                    assert!(!dir.path().join("marker").exists());
                }

                #[test]
                fn run_async_with_sink_failed_should_return_error_without_output() {
                    crate::process::tests::process::run_async_with_sink::run_async_with_sink_failed_should_return_error_without_output(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& echo err 1>&2&& exit 3"),
                    );
                }

                #[test]
                fn run_async_with_sink_failing_sink_should_return_its_error() {
                    crate::process::tests::process::run_async_with_sink::run_async_with_sink_failing_sink_should_return_its_error(
                        &process(std::env::temp_dir()),
                        &shell("echo out&& echo err 1>&2"),
                    );
                }
            }

            mod interceptors {
                use super::*;

                #[test]
                fn interceptors_should_return_std_stream_by_default() {
                    crate::process::tests::process::interceptors::interceptors_should_return_std_stream_by_default(
                        &process(std::env::temp_dir()),
                    );
                }
            }

            mod clear_interceptors {
                use super::*;

                #[test]
                fn clear_interceptors_should_remove_interceptors_of_later_runs() {
                    crate::process::tests::process::clear_interceptors::clear_interceptors_should_remove_interceptors_of_later_runs(
                        &process(std::env::temp_dir()),
                        &shell("exit 0"),
                    );
                }
            }

            mod add_interceptor {
                use super::*;

                #[test]
                fn add_interceptor_should_add_innermost_interceptor() {
                    crate::process::tests::process::add_interceptor::add_interceptor_should_add_innermost_interceptor(
                        &process(std::env::temp_dir()),
                        &shell("exit 3"),
                    );
                }
            }
        }
    }

    mod emitted {
        use super::*;

        #[test]
        fn emitted_should_return_result_of_ready_future() {
            let result = emitted(std::future::ready(Err(crate::Error::process("full"))));

            assert_eq!(result.unwrap_err().description(), "full");
        }

        #[cfg(debug_assertions)]
        #[test]
        #[should_panic(expected = "passing a chunk of a command on should not wait")]
        fn emitted_pending_future_should_panic_in_debug_builds() {
            let _ = emitted(std::future::poll_fn(|context| {
                context.waker().wake_by_ref();
                std::task::Poll::Pending
            }));
        }
    }
}
