use std::collections::VecDeque;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::time::Duration;

use crate::CommandOptions;
use crate::CommandOutput;
use crate::Process;
use crate::ProcessCommand;
use crate::Result;
use crate::env::EnvImpl;
use crate::env::is_same_key;
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
use crate::interceptor::Subject;
use crate::process::io_error;
use crate::process::run_blocking;
use crate::process::std_runs;
use crate::test_platform::ROOT;
use crate::test_platform::gate::Gate;
use crate::test_platform::gate::HeldGate;
#[cfg(feature = "tokio")]
use crate::test_platform::gate::YieldOnce;
use crate::utils::Runs;

/// Enabled by the `testing` feature flag.
///
/// A memory-only [`crate::Process`] implementation for tests. No real
/// process is ever spawned. Commands are matched by their command line
/// (the program and the arguments separated by spaces) against the
/// patterns set with [`TestProcess::set_output`] and
/// [`TestProcess::set_io_error`]. The runs pass through the interceptors
/// like with any [`Process`], [`StdStream`](crate::StdStream) by default,
/// and are recorded when their command would run, i.e. not when they are
/// replayed, e.g. by [`DryRun`](crate::interceptor::DryRun). Delays, e.g.
/// of [`Retry`](crate::interceptor::Retry), are not waited for but
/// recorded, see [`TestProcess::sleeps`]. With the `tokio` feature one is
/// pending once, so that an async run can be cancelled while it waits.
///
/// # Examples
///
/// ```rust
/// use agplatform::{CommandExt, CommandOutput, TestProcess};
/// use std::process::Command;
///
/// let process = TestProcess::new();
/// process.set_output("git status", CommandOutput { status: Some(0), stdout: b"clean".to_vec(), ..Default::default() });
///
/// let output = Command::new("git").arg("status").run_blocking(&process).unwrap();
///
/// assert_eq!(output.stdout_str().unwrap(), "clean");
/// assert_eq!(process.calls(), ["git status"]);
/// ```
pub struct TestProcess {
    env: EnvImpl,
    runs: Runs<ProcessCommand>,
    data: Mutex<TestProcessData>,
}

/// Enabled by the `testing` feature flag.
///
/// A command run by [`TestProcess`], see [`TestProcess::call_details`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestCall {
    /// The program and the arguments separated by spaces, as changed by the
    /// interceptors.
    pub command_line: String,

    /// The directory the command ran in, i.e. its own resolved against the
    /// current directory.
    pub current_dir: PathBuf,

    /// The environment variables the command got: those of the platform,
    /// none with [`CommandOptions::clear_env`], updated by those the
    /// command sets or removes. See [`Self::var`].
    pub vars: Vec<(OsString, OsString)>,

    /// The options of the run.
    pub options: CommandOptions,
}

#[derive(Default)]
struct TestProcessData {
    responses: Vec<(String, VecDeque<TestResponse>)>,
    calls: Vec<TestCall>,
    sleeps: Vec<Duration>,
}

#[derive(Clone)]
struct TestResponse {
    result: std::result::Result<CommandOutput, std::io::ErrorKind>,
    gate: Option<Gate<std::convert::Infallible>>,
}

/// Enabled by the `testing` and `process` feature flags.
///
/// Holds the runs answered by [`TestProcess::set_pending`] until it is
/// released, e.g. to check what happens while a command is running or
/// when its run is cancelled. Clones share the gate, so it can be released
/// from another thread or task.
///
/// Dropping all clones releases it, so that a test failing before it
/// releases the gate does not hang waiting for the held runs, e.g. at the
/// end of [`std::thread::scope`] if it is created in the scope.
#[derive(Clone)]
pub struct TestProcessGate(HeldGate<std::convert::Infallible>);

impl TestCall {
    /// Returns the value of the environment variable `key` the command
    /// got, see [`Self::vars`]. On Windows the key is matched ASCII case
    /// insensitively like in the Windows environment.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandExt, Env};
    /// use std::process::Command;
    ///
    /// let platform = agplatform::test_platform::test_platform();
    /// platform.env.set_var("TOKEN", "secret");
    /// let _ = Command::new("git").env_remove("GIT_DIR").run_blocking(&platform.process);
    ///
    /// let call = &platform.process.call_details()[0];
    /// assert_eq!(call.var("TOKEN"), Some("secret".as_ref()));
    /// assert_eq!(call.var("GIT_DIR"), None);
    /// ```
    pub fn var<K: AsRef<OsStr>>(&self, key: K) -> Option<&OsStr> {
        self.vars
            .iter()
            .find(|(k, _)| is_same_key(k, key.as_ref()))
            .map(|(_, value)| value.as_os_str())
    }
}

impl TestProcess {
    /// Creates a new test process interface whose current directory is
    /// the [`ROOT`] directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::TestProcess;
    ///
    /// let process = TestProcess::new();
    /// ```
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            env: EnvImpl::new(Vec::new(), PathBuf::from(ROOT)),
            runs: std_runs(),
            data: Mutex::new(TestProcessData::default()),
        }
    }

    /// Returns the command lines of all commands run so far, oldest
    /// first.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandExt, TestProcess};
    /// use std::process::Command;
    ///
    /// let process = TestProcess::new();
    /// let _ = Command::new("git").arg("status").run_blocking(&process);
    ///
    /// assert_eq!(process.calls(), ["git status"]);
    /// ```
    pub fn calls(&self) -> Vec<String> {
        self.data()
            .calls
            .iter()
            .map(|call| call.command_line.clone())
            .collect()
    }

    /// Returns all commands run so far with their directory, environment
    /// variables and options, oldest first.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandExt, TestProcess};
    /// use std::process::Command;
    ///
    /// let process = TestProcess::new();
    /// let _ = Command::new("kubectl")
    ///     .current_dir("repo")
    ///     .env("KUBECONFIG", "config")
    ///     .input("manifest")
    ///     .run_blocking(&process);
    ///
    /// let call = &process.call_details()[0];
    /// assert_eq!(call.command_line, "kubectl");
    /// assert!(call.current_dir.ends_with("repo"));
    /// assert_eq!(call.var("KUBECONFIG"), Some("config".as_ref()));
    /// assert_eq!(call.options.input.as_deref(), Some(b"manifest".as_slice()));
    /// ```
    pub fn call_details(&self) -> Vec<TestCall> {
        self.data().calls.clone()
    }

    /// Forgets the commands run so far.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandExt, TestProcess};
    /// use std::process::Command;
    ///
    /// let process = TestProcess::new();
    /// let _ = Command::new("git").arg("status").run_blocking(&process);
    /// process.clear_calls();
    ///
    /// assert!(process.calls().is_empty());
    /// ```
    pub fn clear_calls(&self) {
        self.data().calls.clear();
    }

    /// Returns the delays the interceptors waited for so far, oldest first,
    /// e.g. to check the backoff of [`Retry`](crate::interceptor::Retry).
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
    /// process.add_interceptor(Retry::new(2, Duration::from_secs(1)).backoff(2));
    /// process.set_output("git", CommandOutput { status: Some(1), ..Default::default() });
    ///
    /// let _ = Command::new("git").arg("fetch").run_blocking(&process);
    ///
    /// assert_eq!(process.sleeps(), [Duration::from_secs(1), Duration::from_secs(2)]);
    /// ```
    pub fn sleeps(&self) -> Vec<Duration> {
        self.data().sleeps.clone()
    }

    /// Sets the `output` of the commands whose command line starts with
    /// `pattern`. When more patterns match, the most recently set one
    /// (including by [`Self::set_io_error`]) wins, so an empty pattern
    /// set first acts as a default.
    ///
    /// A [`CommandOutput::duration`] reaching the
    /// [`CommandOptions::timeout`] of a command makes it time out, without
    /// waiting. The output with non-zero status will return an error with
    /// the output attached to it that can be retrieved via [`Error::command_output`](crate::Error::command_output).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandExt, CommandOutput, TestProcess};
    /// use std::process::Command;
    ///
    /// let process = TestProcess::new();
    /// process.set_output("", CommandOutput::default());
    /// process.set_output("git push", CommandOutput { status: Some(1), ..Default::default() });
    ///
    /// assert!(Command::new("git").arg("pull").run_blocking(&process).is_ok());
    /// assert!(Command::new("git").arg("push").run_blocking(&process).is_err());
    /// ```
    pub fn set_output<T: Into<String>>(&self, pattern: T, output: CommandOutput) {
        self.data().responses.push((
            pattern.into(),
            VecDeque::from([TestResponse::new(Ok(output))]),
        ));
    }

    /// Sets the `outputs` of the commands whose command line starts with
    /// `pattern`, returned one per run in order with the last one repeated,
    /// e.g. to make a command fail before it succeeds on a retry. Patterns
    /// match as in [`Self::set_output`]. No `outputs` set nothing.
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
    /// process.set_outputs("curl", [failure, CommandOutput::default()]);
    ///
    /// process.add_interceptor(Retry::new(3, Duration::from_secs(1)));
    ///
    /// let output = Command::new("curl").run_blocking(&process).unwrap();
    ///
    /// assert_eq!(output.status, Some(0));
    /// assert_eq!(process.calls(), ["curl", "curl"]);
    /// ```
    pub fn set_outputs<T: Into<String>, I: IntoIterator<Item = CommandOutput>>(
        &self,
        pattern: T,
        outputs: I,
    ) {
        let outputs: VecDeque<_> = outputs
            .into_iter()
            .map(|output| TestResponse::new(Ok(output)))
            .collect();

        if !outputs.is_empty() {
            self.data().responses.push((pattern.into(), outputs));
        }
    }

    /// Makes the commands whose command line starts with `pattern` fail to
    /// start with an [`std::io::Error`] of `kind`, e.g.
    /// [`NotFound`](std::io::ErrorKind::NotFound) for a program that is not
    /// installed. Patterns match as in [`Self::set_output`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandExt, TestProcess};
    /// use std::io::ErrorKind;
    /// use std::process::Command;
    ///
    /// let process = TestProcess::new();
    /// process.set_io_error("docker", ErrorKind::NotFound);
    ///
    /// let error = Command::new("docker").run_blocking(&process).unwrap_err();
    ///
    /// assert_eq!(error.source_as::<std::io::Error>().unwrap().kind(), ErrorKind::NotFound);
    /// assert!(error.command_output().is_none());
    /// ```
    pub fn set_io_error<T: Into<String>>(&self, pattern: T, kind: std::io::ErrorKind) {
        self.data().responses.push((
            pattern.into(),
            VecDeque::from([TestResponse::new(Err(kind))]),
        ));
    }

    /// Makes the commands whose command line starts with `pattern` return
    /// `output` like [`Self::set_output`], held until the returned gate is
    /// released. The runs are recorded when they start, before being held.
    /// A [`CommandOptions::timeout`] does not end the hold.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandExt, CommandOutput, TestProcess};
    /// use std::process::Command;
    ///
    /// let process = TestProcess::new();
    ///
    /// std::thread::scope(|scope| {
    ///     let gate = process.set_pending("cargo build", CommandOutput::default());
    ///     let building = scope.spawn(|| Command::new("cargo").arg("build").run_blocking(&process));
    ///
    ///     while process.calls().is_empty() {
    ///         std::thread::yield_now();
    ///     }
    ///
    ///     assert!(!building.is_finished());
    ///     gate.release();
    ///     assert!(building.join().unwrap().is_ok());
    /// });
    /// ```
    pub fn set_pending<T: Into<String>>(
        &self,
        pattern: T,
        output: CommandOutput,
    ) -> TestProcessGate {
        let gate = TestProcessGate(HeldGate::new());
        self.data().responses.push((
            pattern.into(),
            VecDeque::from([TestResponse {
                gate: Some(gate.0.gate().clone()),
                ..TestResponse::new(Ok(output))
            }]),
        ));
        gate
    }

    /// [internal]
    ///
    /// Takes the current directory and the variable changes from `env`,
    /// e.g. of a [`TestPlatform`](crate::test_platform::TestPlatform).
    pub(crate) fn set_env(&mut self, env: EnvImpl) {
        self.env = env;
    }

    /// [internal]
    ///
    /// Returns the current directory the commands run in.
    #[cfg(test)]
    pub(crate) fn current_dir(&self) -> PathBuf {
        self.env.current_dir_arc().to_path_buf()
    }

    /// Records the call of `command` and returns the response set for it,
    /// the most recent one matching.
    fn select(&self, command: &ProcessCommand) -> Result<TestResponse> {
        let options = command.options();
        let command_line = command.command_line();
        let mut vars = if options.clear_env {
            Vec::new()
        } else {
            command.vars().unwrap_or_default().to_vec()
        };

        for (key, value) in command.envs() {
            vars.retain(|(k, _)| !is_same_key(k, key));

            if let Some(value) = value {
                vars.push((key.clone(), value.clone()));
            }
        }

        let mut data = self.data();
        data.calls.push(TestCall {
            command_line: command_line.clone(),
            // WHY: only commands resolved against the env are run.
            current_dir: command.current_dir().unwrap_or(Path::new("")).to_path_buf(),
            vars,
            options: options.clone(),
        });
        let response = data
            .responses
            .iter_mut()
            .rev()
            .find(|(pattern, _)| command_line.starts_with(pattern.as_str()))
            .and_then(|(_, responses)| {
                if responses.len() > 1 {
                    responses.pop_front()
                } else {
                    responses.front().cloned()
                }
            })
            .ok_or_else(|| process_error!("'{command_line}': no output set"));
        drop(data);
        response
    }

    fn data(&self) -> std::sync::MutexGuard<'_, TestProcessData> {
        self.data.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl TestResponse {
    fn new(result: std::result::Result<CommandOutput, std::io::ErrorKind>) -> Self {
        Self { result, gate: None }
    }
}

impl TestProcessGate {
    /// Lets the held runs return their output, and the later ones at once.
    /// Does nothing if the gate was released already.
    pub fn release(&self) {
        self.0.gate().open(None);
    }
}

/// Answers the runs of a [`TestProcess`], blocking and async alike: a held
/// response waits for its gate without blocking the thread when the
/// `tokio` feature is enabled, which a blocking run waits for on its
/// thread.
impl Executor for TestProcess {
    type Subject = ProcessCommand;

    fn execute<'a>(
        &'a self,
        run: &'a Run<ProcessCommand>,
        mut emit: Emit<'a, ProcessCommand>,
    ) -> BoxFuture<'a, Result<CommandOutput>> {
        Box::pin(async move {
            let response = self.select(run.subject())?;

            if let Some(gate) = &response.gate {
                #[cfg(feature = "tokio")]
                gate.wait_async().await;
                #[cfg(not(feature = "tokio"))]
                gate.wait();
            }

            respond(run, response.result, &mut emit).await
        })
    }

    /// Delays are recorded instead of waited for, like timeouts.
    fn sleep(&self, delay: Duration) -> BoxFuture<'static, ()> {
        self.data().sleeps.push(delay);
        #[cfg(feature = "tokio")]
        return Box::pin(YieldOnce(false));
        #[cfg(not(feature = "tokio"))]
        Box::pin(std::future::ready(()))
    }
}

impl BlockingExecutor for TestProcess {}

#[cfg(feature = "tokio")]
impl AsyncExecutor for TestProcess {}

impl Process for TestProcess {
    /// Records the command and returns the output set for it, without
    /// reading the input. Its non-empty output is passed to the
    /// interceptors at once. With [`CommandOptions::silent`] or
    /// [`CommandOptions::interactive`] the output set is dropped, as no
    /// output would be captured.
    ///
    /// See [`crate::Process::run_blocking`].
    ///
    /// # Errors
    ///
    /// Returns an error if no output was set for the command, an I/O error
    /// was set for it, or as [`crate::Process::run_blocking`] for the options and
    /// the output set.
    fn run_blocking(&self, run: impl Into<Run<ProcessCommand>>) -> Result<CommandOutput> {
        run_blocking(&self.runs, &self.env, self, run.into(), None)
    }

    /// Runs the command like [`Self::run_blocking`], passing the output set for it to
    /// `sink`, the standard output and then the standard error, each in one
    /// chunk unless empty.
    ///
    /// See [`crate::Process::run_blocking_with_sink`].
    ///
    /// # Errors
    ///
    /// As [`Self::run_blocking`], or as [`crate::Process::run_blocking_with_sink`].
    fn run_blocking_with_sink(
        &self,
        run: impl Into<Run<ProcessCommand>>,
        sink: &mut dyn Sink<ProcessCommand>,
    ) -> Result<CommandOutput> {
        run_blocking(&self.runs, &self.env, self, run.into(), Some(sink))
    }

    /// Runs the command like [`Self::run_blocking`] once awaited, holding only the
    /// task while a [`Self::set_pending`] gate is closed. Dropping the
    /// future cancels the run, which is told to the interceptors.
    ///
    /// See [`crate::Process::run_async`].
    ///
    /// # Errors
    ///
    /// As [`Self::run_blocking`].
    #[cfg(feature = "tokio")]
    fn run_async(
        &self,
        run: impl Into<Run<ProcessCommand>>,
    ) -> impl Future<Output = Result<CommandOutput>> + Send {
        crate::process::run_async(&self.runs, &self.env, self, run.into(), None)
    }

    /// Runs the command like [`Self::run_async`], passing the output set for
    /// it to `sink` like [`Self::run_blocking_with_sink`].
    ///
    /// See [`crate::Process::run_async_with_sink`].
    ///
    /// # Errors
    ///
    /// As [`Self::run_blocking_with_sink`].
    #[cfg(feature = "tokio")]
    fn run_async_with_sink(
        &self,
        run: impl Into<Run<ProcessCommand>>,
        sink: &mut dyn Sink<ProcessCommand>,
    ) -> impl Future<Output = Result<CommandOutput>> + Send {
        crate::process::run_async(&self.runs, &self.env, self, run.into(), Some(sink))
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

/// Passes the output of `run` from the `result` set for it on like a
/// replayed one, see [`Subject::replay`], and returns it timed out if it
/// took as long as the [`CommandOptions::timeout`].
async fn respond(
    run: &Run<ProcessCommand>,
    result: std::result::Result<CommandOutput, std::io::ErrorKind>,
    emit: &mut Emit<'_, ProcessCommand>,
) -> Result<CommandOutput> {
    let command = run.subject();
    let output = result.map_err(|kind| io_error(&command.program(), kind.into()))?;
    ProcessCommand::replay(run, &output, emit).await?;

    Ok(match command.options().timeout {
        Some(timeout) if output.duration >= timeout => CommandOutput {
            status: None,
            duration: timeout,
            timed_out: true,
            ..output
        },
        _ => output,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CommandExt;
    use crate::Env;
    use crate::OutputStream;
    use crate::interceptor::Chunk;
    use crate::interceptor::DryRun;
    use crate::interceptor::Next;
    use crate::interceptor::Retry;
    use std::process::Command;
    use std::sync::Arc;
    use std::time::Duration;

    /// Changes of the env the process shares, like by its platform.
    impl TestProcess {
        fn set_var(&self, key: String, value: String) {
            self.env.set_var(key, value).unwrap();
        }
    }

    fn exit(status: i32) -> CommandOutput {
        CommandOutput {
            status: Some(status),
            ..Default::default()
        }
    }

    /// Returns the output printing `out` and `err` and exiting with
    /// `status`.
    fn printed(status: i32) -> CommandOutput {
        CommandOutput {
            stdout: b"out\n".to_vec(),
            stderr: b"err\n".to_vec(),
            ..exit(status)
        }
    }

    /// Returns the output printing `out` and exiting with code 0 after
    /// `duration`.
    fn slow(duration: Duration) -> CommandOutput {
        CommandOutput {
            stdout: b"out\n".to_vec(),
            duration,
            ..exit(0)
        }
    }

    /// Returns a test process answering `git` with `output`.
    fn answering(output: CommandOutput) -> TestProcess {
        let process = TestProcess::new();
        process.set_output("git", output);
        process
    }

    #[cfg(unix)]
    fn killed() -> CommandOutput {
        CommandOutput {
            status: None,
            signal: Some(9),
            ..Default::default()
        }
    }

    fn missing() -> TestProcess {
        let process = TestProcess::new();
        process.set_io_error("git", std::io::ErrorKind::NotFound);
        process
    }

    #[cfg(feature = "tokio")]
    fn block_on<T>(future: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(future)
    }

    /// Records the runs it handles and the output passing through it.
    #[derive(Clone, Default)]
    struct Recorder(Arc<Mutex<Vec<String>>>);

    impl Recorder {
        fn events(&self) -> Vec<String> {
            self.0.lock().unwrap().clone()
        }
    }

    impl Interceptor<ProcessCommand> for Recorder {
        fn handle<'a>(
            &'a self,
            mut next: Next<'a, ProcessCommand>,
        ) -> BoxFuture<'a, Result<CommandOutput>> {
            Box::pin(async move {
                let command = next.run().subject();
                self.0.lock().unwrap().push(format!(
                    "started {} in {}",
                    command.command_line(),
                    command.current_dir().unwrap().display()
                ));
                let result = next.attempt().await;
                self.0.lock().unwrap().push(format!(
                    "finished {}",
                    result.as_ref().is_ok_and(|output| output.success)
                ));
                result
            })
        }

        fn chunk(&self, chunk: Chunk<ProcessCommand>) -> Result {
            self.0.lock().unwrap().push(format!(
                "{:?} {}",
                chunk.stream(),
                String::from_utf8_lossy(chunk.data())
            ));
            chunk.forward()
        }

        fn cancelled(&self, run: &Run<ProcessCommand>) {
            self.0
                .lock()
                .unwrap()
                .push(format!("cancelled {}", run.subject().program()));
        }
    }

    /// Returns a test process whose only interceptor is the returned
    /// recorder.
    fn observed() -> (TestProcess, Recorder) {
        let recorder = Recorder::default();
        let process = TestProcess::new();
        process.clear_interceptors();
        process.add_interceptor(recorder.clone());
        (process, recorder)
    }

    mod test_call {
        use super::*;

        mod var {
            use super::*;

            fn call(vars: Vec<(OsString, OsString)>) -> TestCall {
                TestCall {
                    command_line: String::new(),
                    current_dir: PathBuf::new(),
                    vars,
                    options: CommandOptions::default(),
                }
            }

            #[test]
            fn var_should_return_value_of_key() {
                let call = call(vec![("A".into(), "1".into()), ("B".into(), "2".into())]);

                assert_eq!(call.var("B"), Some(OsStr::new("2")));
                assert_eq!(call.var("C"), None);
            }

            #[cfg(windows)]
            #[test]
            fn var_should_match_key_ignoring_ascii_case() {
                let call = call(vec![("Path".into(), "1".into())]);

                assert_eq!(call.var("PATH"), Some(OsStr::new("1")));
            }

            #[cfg(unix)]
            #[test]
            fn var_should_match_key_case_sensitively() {
                let call = call(vec![("Path".into(), "1".into())]);

                assert_eq!(call.var("PATH"), None);
            }
        }
    }

    mod test_process {
        use super::*;

        mod calls {
            use super::*;

            #[test]
            fn calls_should_return_command_lines_in_order() {
                let process = TestProcess::new();
                process.set_output("", exit(0));

                Command::new("git")
                    .arg("pull")
                    .run_blocking(&process)
                    .unwrap();
                Command::new("git")
                    .arg("push")
                    .run_blocking(&process)
                    .unwrap();

                assert_eq!(process.calls(), ["git pull", "git push"]);
            }
        }

        mod call_details {
            use super::*;

            #[test]
            fn call_details_should_return_dir_vars_and_options() {
                // GIVEN platform variables
                let process = TestProcess::new();
                process.set_output("", exit(0));
                process.set_var("PLATFORM".to_string(), "1".to_string());
                process.set_var("OVERRIDDEN".to_string(), "1".to_string());
                process.set_var("KEPT".to_string(), "1".to_string());

                // WHEN running a command changing variables in a directory
                Command::new("sort")
                    .current_dir("repo")
                    .env("OVERRIDDEN", "2")
                    .env_remove("PLATFORM")
                    .input("b\na")
                    .run_blocking(&process)
                    .unwrap();

                // THEN the call has the resolved directory and the variables
                assert_eq!(
                    process.call_details(),
                    [TestCall {
                        command_line: "sort".to_string(),
                        current_dir: PathBuf::from(ROOT).join("repo"),
                        vars: vec![
                            ("KEPT".into(), "1".into()),
                            ("OVERRIDDEN".into(), "2".into()),
                        ],
                        options: CommandOptions {
                            input: Some(b"b\na".to_vec()),
                            ..Default::default()
                        },
                    }]
                );
            }

            #[test]
            fn call_details_clear_env_should_have_only_command_vars() {
                let process = TestProcess::new();
                process.set_output("", exit(0));
                process.set_var("PLATFORM".to_string(), "1".to_string());

                Command::new("git")
                    .env("OWN", "1")
                    .clear_env()
                    .run_blocking(&process)
                    .unwrap();

                assert_eq!(process.call_details()[0].vars, [("OWN".into(), "1".into())]);
            }

            #[cfg(windows)]
            #[test]
            fn call_details_should_merge_vars_ignoring_case() {
                let process = TestProcess::new();
                process.set_output("", exit(0));
                process.set_var("Path".to_string(), "platform".to_string());

                Command::new("git")
                    .env("PATH", "own")
                    .run_blocking(&process)
                    .unwrap();

                assert_eq!(
                    process.call_details()[0].vars,
                    [("PATH".into(), "own".into())]
                );
            }
        }

        mod clear_calls {
            use super::*;

            #[test]
            fn clear_calls_should_forget_calls() {
                let process = TestProcess::new();
                process.set_output("", exit(0));
                Command::new("git").run_blocking(&process).unwrap();

                process.clear_calls();

                assert!(process.calls().is_empty());
            }
        }

        mod sleeps {
            use super::*;

            /// Returns the command `git` retried twice after 1 s doubled
            /// by the interceptors after `recorder`.
            fn retried(recorder: &Recorder) -> Run<ProcessCommand> {
                let mut interceptors = Interceptors::new();
                interceptors.push(recorder.clone());
                interceptors.push(Retry::new(2, Duration::from_secs(1)).backoff(2));
                Command::new("git").interceptors(interceptors)
            }

            #[test]
            fn sleeps_should_return_delays_in_order() {
                let (process, recorder) = observed();
                process.set_output("git", exit(1));

                let _ = retried(&recorder).run_blocking(&process);

                assert_eq!(
                    process.sleeps(),
                    [Duration::from_secs(1), Duration::from_secs(2)]
                );
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn sleeps_should_return_async_delays_in_order() {
                let (process, recorder) = observed();
                process.set_output("git", exit(1));

                let _ = block_on(retried(&recorder).run_async(&process));

                assert_eq!(
                    process.sleeps(),
                    [Duration::from_secs(1), Duration::from_secs(2)]
                );
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn sleeps_cancelled_should_tell_interceptors() {
                // GIVEN an async run waiting to be retried
                let (process, recorder) = observed();
                process.set_output("git", exit(1));
                let mut future = Box::pin(retried(&recorder).run_async(&process));
                let mut context = std::task::Context::from_waker(std::task::Waker::noop());
                assert!(future.as_mut().poll(&mut context).is_pending());

                // WHEN cancelling it
                drop(future);

                // THEN the interceptors up to the retry are told after the
                // delay started
                assert_eq!(process.sleeps(), [Duration::from_secs(1)]);
                assert_eq!(recorder.events().last().unwrap(), "cancelled git");
            }
        }

        mod set_output {
            use super::*;

            #[test]
            fn set_output_should_prefer_most_recent_matching_pattern() {
                // GIVEN a specific pattern followed by a more general one
                let process = TestProcess::new();
                process.set_output("git push", exit(1));
                process.set_output("git", exit(0));

                // WHEN running a command matching both
                let output = Command::new("git").arg("push").run_blocking(&process);

                // THEN the more recent one wins
                assert_eq!(output.unwrap(), exit(0));
            }
        }

        mod set_outputs {
            use super::*;

            #[test]
            fn set_outputs_should_return_outputs_in_order_repeating_last() {
                let process = TestProcess::new();
                process.set_outputs("git", vec![exit(1), exit(2), exit(0)]);

                let statuses: Vec<_> = (0..4)
                    .map(|_| match Command::new("git").run_blocking(&process) {
                        Ok(output) => output.status,
                        Err(error) => error.command_output().unwrap().status,
                    })
                    .collect();

                assert_eq!(statuses, [Some(1), Some(2), Some(0), Some(0)]);
            }

            #[test]
            fn set_outputs_empty_should_set_nothing() {
                let process = TestProcess::new();
                process.set_output("", exit(0));

                process.set_outputs("git", Vec::new());

                assert!(Command::new("git").run_blocking(&process).is_ok());
            }

            #[test]
            fn set_outputs_with_retries_should_succeed_without_waiting() {
                // GIVEN a command failing twice before it succeeds
                let process = TestProcess::new();
                process.set_outputs("curl", vec![exit(7), exit(7), exit(0)]);
                let start = std::time::Instant::now();

                // WHEN retrying it with a long delay
                process.add_interceptor(Retry::new(2, Duration::from_secs(3600)));
                let output = Command::new("curl").run_blocking(&process).unwrap();

                // THEN the last attempt succeeds at once
                assert_eq!(output.status, Some(0));
                assert_eq!(process.calls(), ["curl", "curl", "curl"]);
                assert!(start.elapsed() < Duration::from_secs(60));
            }
        }

        mod set_io_error {
            use super::*;

            #[test]
            fn set_io_error_should_fail_run_with_io_source_naming_program() {
                let process = TestProcess::new();
                process.set_io_error("docker", std::io::ErrorKind::NotFound);

                let error = Command::new("docker")
                    .arg("version")
                    .run_blocking(&process)
                    .unwrap_err();

                assert_eq!(error.kind(), crate::ErrorKind::Process);
                assert!(error.description().starts_with("'docker': "));
                assert_eq!(
                    error
                        .source_as::<std::io::Error>()
                        .map(std::io::Error::kind),
                    Some(std::io::ErrorKind::NotFound)
                );
                assert!(error.command_output().is_none());
                assert_eq!(process.calls(), ["docker version"]);
            }

            #[test]
            fn set_io_error_should_override_earlier_matching_output() {
                // GIVEN a default output followed by an I/O error
                let process = TestProcess::new();
                process.set_output("", exit(0));
                process.set_io_error("docker", std::io::ErrorKind::PermissionDenied);

                // WHEN running a command matching both
                let result = Command::new("docker").run_blocking(&process);

                // THEN the more recent I/O error wins
                assert_eq!(
                    result
                        .unwrap_err()
                        .source_as::<std::io::Error>()
                        .map(std::io::Error::kind),
                    Some(std::io::ErrorKind::PermissionDenied)
                );
            }
        }

        mod set_pending {
            use super::*;

            #[test]
            fn set_pending_gate_dropped_should_release_run() {
                let process = TestProcess::new();
                let command = Command::new("cargo");

                std::thread::scope(|scope| {
                    let gate = process.set_pending("cargo", exit(0));
                    let running = scope.spawn(|| command.run_blocking(&process));
                    let _ = std::iter::repeat_with(std::thread::yield_now)
                        .find(|()| !process.calls().is_empty());

                    drop(gate);

                    assert_eq!(running.join().unwrap().unwrap(), exit(0));
                });
            }

            #[test]
            fn set_pending_should_hold_run_until_released() {
                // GIVEN a pending output
                let process = TestProcess::new();
                let gate = process.set_pending("cargo", exit(0));
                let command = Command::new("cargo");

                std::thread::scope(|scope| {
                    // WHEN running a command matching it
                    let running = scope.spawn(|| command.run_blocking(&process));

                    // THEN it is recorded and held until the gate is released
                    let _ = std::iter::repeat_with(std::thread::yield_now)
                        .find(|()| !process.calls().is_empty());
                    assert!(!running.is_finished());
                    gate.release();
                    assert_eq!(running.join().unwrap().unwrap(), exit(0));
                });
            }

            #[test]
            fn set_pending_released_should_not_hold_later_runs() {
                let process = TestProcess::new();
                let gate = process.set_pending("cargo", exit(0));

                gate.release();

                assert_eq!(
                    Command::new("cargo").run_blocking(&process).unwrap(),
                    exit(0)
                );
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn set_pending_should_hold_async_run_until_released() {
                // GIVEN an async run matching a pending output
                let process = TestProcess::new();
                let gate = process.set_pending("cargo", exit(0));
                let command = Command::new("cargo");
                let mut future = std::pin::pin!(command.run_async(&process));
                let mut context = std::task::Context::from_waker(std::task::Waker::noop());

                // WHEN polling it before and after releasing the gate
                let held = future.as_mut().poll(&mut context);
                gate.release();
                let result = block_on(future);

                // THEN it is recorded and held until released
                assert!(held.is_pending());
                assert_eq!(process.calls(), ["cargo"]);
                assert_eq!(result.unwrap(), exit(0));
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn set_pending_released_should_not_hold_later_async_runs() {
                let process = TestProcess::new();
                let gate = process.set_pending("cargo", exit(0));

                gate.release();

                assert_eq!(
                    block_on(Command::new("cargo").run_async(&process)).unwrap(),
                    exit(0)
                );
            }

            #[cfg(feature = "tokio")]
            #[test]
            fn set_pending_cancelled_should_tell_interceptors() {
                // GIVEN an async run held by a pending output
                let (process, recorder) = observed();
                let _gate = process.set_pending("cargo", exit(0));
                let command = Command::new("cargo");
                let mut future = Box::pin(command.run_async(&process));
                let mut context = std::task::Context::from_waker(std::task::Waker::noop());
                assert!(future.as_mut().poll(&mut context).is_pending());

                // WHEN cancelling it
                drop(future);

                // THEN the interceptors are told
                assert_eq!(
                    recorder.events(),
                    [
                        format!("started cargo in {ROOT}"),
                        "cancelled cargo".to_string()
                    ]
                );
            }
        }

        mod process {
            use super::*;

            mod run_blocking {
                use super::*;

                #[test]
                fn run_blocking_should_capture_output() {
                    crate::process::tests::process::run_blocking::run_blocking_should_capture_output(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_env_changed_while_retrying_should_keep_env_of_run() {
                    // GIVEN an interceptor changing the env after each attempt
                    struct Changing(EnvImpl);

                    impl Interceptor<ProcessCommand> for Changing {
                        fn handle<'a>(
                            &'a self,
                            mut next: Next<'a, ProcessCommand>,
                        ) -> BoxFuture<'a, Result<CommandOutput>> {
                            Box::pin(async move {
                                let result = next.attempt().await;
                                self.0.set_var("A", "2").unwrap();
                                self.0.replace_current_dir(PathBuf::from("/other"));
                                result
                            })
                        }
                    }

                    let process = TestProcess::new();
                    process.set_var("A".to_string(), "1".to_string());
                    process.add_interceptor(Retry::new(1, Duration::ZERO));
                    process.add_interceptor(Changing(process.env.clone()));
                    process.set_outputs("curl", vec![exit(7), exit(0)]);

                    // WHEN running a command retried once
                    Command::new("curl").run_blocking(&process).unwrap();

                    // THEN both attempts get the env the run started with
                    let calls = process.call_details();
                    assert_eq!(calls.len(), 2);
                    for call in calls {
                        assert_eq!(call.vars, [("A".into(), "1".into())]);
                        assert_eq!(call.current_dir, Path::new(ROOT));
                    }
                }

                #[test]
                fn run_blocking_unmatched_should_return_error_and_record_call() {
                    let process = TestProcess::new();
                    process.set_output("cargo", exit(0));

                    let error = Command::new("git").run_blocking(&process).unwrap_err();

                    assert_eq!(error.description(), "'git': no output set");
                    assert_eq!(process.calls(), ["git"]);
                }

                #[test]
                fn run_blocking_invalid_options_should_return_error() {
                    let process = answering(exit(0));

                    crate::process::tests::process::run_blocking::run_blocking_invalid_options_should_return_error(
                        &process,
                        &Command::new("git"),
                    );

                    assert!(process.calls().is_empty());
                }

                #[test]
                fn run_blocking_non_zero_exit_should_return_error_with_output() {
                    crate::process::tests::process::run_blocking::run_blocking_non_zero_exit_should_return_error_with_output(
                        &answering(printed(3)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_accepted_timeout_should_return_output() {
                    struct AcceptTimeout;

                    impl Interceptor<ProcessCommand> for AcceptTimeout {
                        fn handle<'a>(
                            &'a self,
                            mut next: Next<'a, ProcessCommand>,
                        ) -> BoxFuture<'a, Result<CommandOutput>> {
                            Box::pin(async move {
                                next.attempt().await.map(|output| CommandOutput {
                                    success: output.timed_out,
                                    ..output
                                })
                            })
                        }
                    }

                    let process = TestProcess::new();
                    process.add_interceptor(AcceptTimeout);
                    process.set_output(
                        "git",
                        CommandOutput {
                            duration: Duration::from_secs(60),
                            ..exit(0)
                        },
                    );

                    let output = Command::new("git")
                        .timeout(Duration::from_secs(1))
                        .run_blocking(&process)
                        .unwrap();

                    assert!(output.timed_out);
                    assert_eq!(output.duration, Duration::from_secs(1));
                }

                #[test]
                fn run_blocking_zero_timeout_should_time_out() {
                    crate::process::tests::process::run_blocking::run_blocking_zero_timeout_should_time_out(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_exceeding_timeout_should_time_out_keeping_output() {
                    crate::process::tests::process::run_blocking::run_blocking_exceeding_timeout_should_time_out_keeping_output(
                        &answering(slow(Duration::from_secs(60))),
                        &Command::new("git"),
                        Duration::from_secs(1),
                    );
                }

                #[test]
                fn run_blocking_display_lines_should_return_output_equal_to_set() {
                    let process = TestProcess::new();
                    let output = CommandOutput {
                        stdout: b"out".to_vec(),
                        ..exit(0)
                    };
                    process.set_output("git", output.clone());

                    let result = Command::new("git")
                        .display_lines(3)
                        .run_blocking(&process)
                        .unwrap();

                    assert_eq!(result, output);
                    assert_eq!(result.display_lines, 3);
                }

                #[test]
                fn run_blocking_within_timeout_should_succeed() {
                    crate::process::tests::process::run_blocking::run_blocking_within_timeout_should_succeed(
                        &answering(slow(Duration::from_secs(1))),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_silent_should_neither_capture_nor_stream_output() {
                    crate::process::tests::process::run_blocking::run_blocking_silent_should_neither_capture_nor_stream_output(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_streaming_should_pass_output_to_interceptors() {
                    crate::process::tests::process::run_blocking::run_blocking_streaming_should_pass_output_to_interceptors(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn run_blocking_killed_by_signal_should_report_signal() {
                    crate::process::tests::process::run_blocking::run_blocking_killed_by_signal_should_report_signal(
                        &answering(killed()),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_missing_program_should_return_error_with_io_source() {
                    crate::process::tests::process::run_blocking::run_blocking_missing_program_should_return_error_with_io_source(
                        &missing(),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_maximum_timeout_should_not_limit_run() {
                    crate::process::tests::process::run_blocking::run_blocking_maximum_timeout_should_not_limit_run(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_interactive_should_not_capture_output() {
                    crate::process::tests::process::run_blocking::run_blocking_interactive_should_not_capture_output(
                        &answering(printed(3)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_should_pass_run_and_result_to_interceptors() {
                    let process = answering(exit(0));
                    process.set_output("cargo", exit(3));

                    crate::process::tests::process::run_blocking::run_blocking_should_pass_run_and_result_to_interceptors(
                        &process,
                        &Command::new("git"),
                        &Command::new("cargo"),
                        Path::new(ROOT),
                    );
                }

                #[test]
                fn run_blocking_program_changed_by_interceptor_should_answer_changed_command() {
                    // GIVEN an interceptor changing the program
                    struct Rename;

                    impl Interceptor<ProcessCommand> for Rename {
                        fn handle<'a>(
                            &'a self,
                            mut next: Next<'a, ProcessCommand>,
                        ) -> BoxFuture<'a, Result<CommandOutput>> {
                            next.run_mut().subject_mut().set_program("hg");
                            next.forward()
                        }
                    }

                    let process = TestProcess::new();
                    process.add_interceptor(Rename);
                    process.set_output("git", exit(1));
                    process.set_output("hg", exit(0));

                    // WHEN running a command
                    let output = Command::new("git").arg("log").run_blocking(&process);

                    // THEN the changed command is answered and recorded
                    assert_eq!(output.unwrap(), exit(0));
                    assert_eq!(process.calls(), ["hg log"]);
                }
            }

            #[cfg(feature = "tokio")]
            mod run_async {
                use super::*;

                #[test]
                fn run_async_should_capture_output() {
                    crate::process::tests::process::run_async::run_async_should_capture_output(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_async_unmatched_should_fail() {
                    let process = TestProcess::new();
                    let command = Command::new("git");

                    let error = block_on(command.run_async(&process)).unwrap_err();

                    assert_eq!(error.description(), "'git': no output set");
                }

                #[test]
                fn run_async_should_retry_without_waiting() {
                    let process = TestProcess::new();
                    process.set_outputs("git", [exit(1), exit(0)]);
                    let command = Command::new("git");

                    process.add_interceptor(Retry::new(1, Duration::from_secs(3600)));

                    let result = block_on(command.run_async(&process));

                    assert_eq!(result.unwrap(), exit(0));
                    assert_eq!(process.calls(), ["git", "git"]);
                }

                #[test]
                fn run_async_invalid_options_should_return_error() {
                    let process = answering(exit(0));

                    crate::process::tests::process::run_async::run_async_invalid_options_should_return_error(
                        &process,
                        &Command::new("git"),
                    );

                    assert!(process.calls().is_empty());
                }

                #[test]
                fn run_async_panicking_interceptor_should_not_tell_cancelled() {
                    // GIVEN a recorder outside an interceptor panicking on the
                    // output of a streamed run
                    struct Panicking;

                    impl Interceptor<ProcessCommand> for Panicking {
                        fn chunk(&self, chunk: Chunk<ProcessCommand>) -> Result {
                            if chunk.run().subject().options().stream {
                                panic!("interceptor");
                            }

                            chunk.forward()
                        }
                    }

                    let (process, recorder) = observed();
                    process.add_interceptor(Panicking);
                    process.set_output(
                        "cargo",
                        CommandOutput {
                            stdout: b"output".to_vec(),
                            ..exit(0)
                        },
                    );
                    let command = Command::new("cargo");
                    block_on(command.run_async(&process)).unwrap();

                    // WHEN running it streamed
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        block_on(command.stream().run_async(&process))
                    }));

                    // THEN the run is not told as cancelled
                    assert!(result.is_err());
                    assert_eq!(
                        recorder.events(),
                        [
                            format!("started cargo in {ROOT}"),
                            "Stdout output".to_string(),
                            "finished true".to_string(),
                            format!("started cargo in {ROOT}")
                        ]
                    );
                }

                #[test]
                fn run_async_non_zero_exit_should_return_error_with_output() {
                    crate::process::tests::process::run_async::run_async_non_zero_exit_should_return_error_with_output(
                        &answering(printed(3)),
                        &Command::new("git"),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn run_async_killed_by_signal_should_report_signal() {
                    crate::process::tests::process::run_async::run_async_killed_by_signal_should_report_signal(
                        &answering(killed()),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_async_missing_program_should_return_error_with_io_source() {
                    crate::process::tests::process::run_async::run_async_missing_program_should_return_error_with_io_source(
                        &missing(),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_async_within_timeout_should_succeed() {
                    crate::process::tests::process::run_async::run_async_within_timeout_should_succeed(
                        &answering(slow(Duration::from_secs(1))),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_async_maximum_timeout_should_not_limit_run() {
                    crate::process::tests::process::run_async::run_async_maximum_timeout_should_not_limit_run(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_async_zero_timeout_should_time_out() {
                    crate::process::tests::process::run_async::run_async_zero_timeout_should_time_out(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_async_exceeding_timeout_should_time_out_keeping_output() {
                    crate::process::tests::process::run_async::run_async_exceeding_timeout_should_time_out_keeping_output(
                        &answering(slow(Duration::from_secs(60))),
                        &Command::new("git"),
                        Duration::from_secs(1),
                    );
                }

                #[test]
                fn run_async_silent_should_neither_capture_nor_stream_output() {
                    crate::process::tests::process::run_async::run_async_silent_should_neither_capture_nor_stream_output(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_async_interactive_should_not_capture_output() {
                    crate::process::tests::process::run_async::run_async_interactive_should_not_capture_output(
                        &answering(printed(3)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_async_streaming_should_pass_output_to_interceptors() {
                    crate::process::tests::process::run_async::run_async_streaming_should_pass_output_to_interceptors(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_async_cancelled_should_tell_interceptors() {
                    let process = TestProcess::new();
                    let _gate = process.set_pending("git", exit(0));

                    crate::process::tests::process::run_async::run_async_cancelled_should_tell_interceptors(
                        &process,
                        &Command::new("git"),
                    );
                }
            }

            mod run_blocking_with_sink {
                use super::*;

                fn output() -> CommandOutput {
                    CommandOutput {
                        status: Some(0),
                        stdout: b"out".to_vec(),
                        stderr: b"err".to_vec(),
                        ..Default::default()
                    }
                }

                #[test]
                fn run_blocking_with_sink_should_pass_output_set_to_sink_by_stream() {
                    let process = TestProcess::new();
                    process.set_output("git", output());
                    let mut sunk = Vec::new();

                    let result = Command::new("git").run_blocking_with_sink(
                        &process,
                        &mut |stream: OutputStream, data: &[u8]| {
                            sunk.push((stream, data.to_vec()));
                            Ok(())
                        },
                    );

                    assert_eq!(
                        result.unwrap(),
                        CommandOutput {
                            stdout: Vec::new(),
                            stderr: Vec::new(),
                            ..output()
                        }
                    );
                    assert_eq!(
                        sunk,
                        [
                            (OutputStream::Stdout, b"out".to_vec()),
                            (OutputStream::Stderr, b"err".to_vec())
                        ]
                    );
                }

                #[test]
                fn run_blocking_with_sink_failed_without_output_should_be_retried() {
                    let process = TestProcess::new();
                    process.set_outputs("git", [exit(1), output()]);
                    let mut sunk = Vec::new();

                    process.add_interceptor(Retry::new(1, Duration::from_secs(60)));

                    let result = Command::new("git").run_blocking_with_sink(
                        &process,
                        &mut |_: OutputStream, data: &[u8]| {
                            sunk.extend_from_slice(data);
                            Ok(())
                        },
                    );

                    assert_eq!(result.unwrap().status, Some(0));
                    assert_eq!(sunk, b"outerr");
                    assert_eq!(process.calls(), ["git", "git"]);
                }

                #[test]
                fn run_blocking_with_sink_failing_sink_should_return_its_error() {
                    crate::process::tests::process::run_blocking_with_sink::run_blocking_with_sink_failing_sink_should_return_its_error(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_with_sink_should_pass_output_to_sink_instead_of_capturing() {
                    crate::process::tests::process::run_blocking_with_sink::run_blocking_with_sink_should_pass_output_to_sink_instead_of_capturing(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_with_sink_should_pass_output_through_interceptors_to_sink() {
                    crate::process::tests::process::run_blocking_with_sink::run_blocking_with_sink_should_pass_output_through_interceptors_to_sink(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_with_sink_failed_should_return_error_without_output() {
                    crate::process::tests::process::run_blocking_with_sink::run_blocking_with_sink_failed_should_return_error_without_output(
                        &answering(printed(3)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_blocking_with_sink_silent_should_fail_without_running() {
                    crate::process::tests::process::run_blocking_with_sink::run_blocking_with_sink_silent_should_fail_without_running(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }
            }

            #[cfg(feature = "tokio")]
            mod run_async_with_sink {
                use super::*;

                #[test]
                fn run_async_with_sink_should_pass_output_to_sink_instead_of_capturing() {
                    crate::process::tests::process::run_async_with_sink::run_async_with_sink_should_pass_output_to_sink_instead_of_capturing(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_async_with_sink_failed_should_return_error_without_output() {
                    crate::process::tests::process::run_async_with_sink::run_async_with_sink_failed_should_return_error_without_output(
                        &answering(printed(3)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn run_async_with_sink_failing_sink_should_return_its_error() {
                    crate::process::tests::process::run_async_with_sink::run_async_with_sink_failing_sink_should_return_its_error(
                        &answering(printed(0)),
                        &Command::new("git"),
                    );
                }
            }

            mod interceptors {
                use super::*;

                #[test]
                fn interceptors_should_return_std_stream_by_default() {
                    crate::process::tests::process::interceptors::interceptors_should_return_std_stream_by_default(
                        &TestProcess::new(),
                    );
                }
            }

            mod clear_interceptors {
                use super::*;

                #[test]
                fn clear_interceptors_should_remove_interceptors_of_later_runs() {
                    crate::process::tests::process::clear_interceptors::clear_interceptors_should_remove_interceptors_of_later_runs(
                        &answering(exit(0)),
                        &Command::new("git"),
                    );
                }
            }

            mod add_interceptor {
                use super::*;

                #[test]
                fn add_interceptor_should_add_innermost_interceptor() {
                    crate::process::tests::process::add_interceptor::add_interceptor_should_add_innermost_interceptor(
                        &answering(exit(3)),
                        &Command::new("git"),
                    );
                }

                #[test]
                fn add_interceptor_should_pass_runs_in_resolved_dir() {
                    let (process, recorder) = observed();

                    let _ = Command::new("git")
                        .current_dir("repo")
                        .run_blocking(&process);

                    assert_eq!(
                        recorder.events(),
                        [
                            format!(
                                "started git in {}",
                                PathBuf::from(ROOT).join("repo").display()
                            ),
                            "finished false".to_string()
                        ]
                    );
                }

                #[test]
                fn add_interceptor_setting_output_should_not_record_call() {
                    let process = TestProcess::new();
                    process.add_interceptor(DryRun);

                    let output = Command::new("kubectl").run_blocking(&process).unwrap();

                    assert_eq!(output, exit(0));
                    assert!(process.calls().is_empty());
                }
            }
        }
    }
}
