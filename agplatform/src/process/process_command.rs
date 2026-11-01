use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use crate::Result;
use crate::env::EnvImpl;
use crate::interceptor::BoxFuture;
use crate::interceptor::Captured;
use crate::interceptor::Emit;
use crate::interceptor::Interceptors;
use crate::interceptor::Retry;
use crate::interceptor::Run;
use crate::interceptor::Sink;
use crate::interceptor::Subject;
use crate::utils::OwnInterceptors;

use super::CommandOutput;
use super::DISPLAY_LINES;
use super::OutputStream;
use super::Process;

/// A command run by a [`Process`] with its [`CommandOptions`], the [`Subject`]
/// of the [`Interceptor`](crate::interceptor::Interceptor)s of the runs of
/// commands.
///
/// Holds a copy of the program, the arguments, the environment variables and
/// the current directory of a [`Command`], which is left unchanged. Usually run
/// as a [`Run`] from [`CommandExt`](crate::CommandExt), passed to the
/// interceptors with the directory the command runs in, which may change it,
/// see [`Self::current_dir`].
///
/// The interceptors of a [`Process`] are [`StdStream`](crate::StdStream) by
/// default, see [`Process::add_interceptor`], e.g. on
/// [`Platform::process`](crate::Platform::process). A run uses them unless it
/// has its own, see [`Run::interceptors`](Run#method.interceptors). A failed
/// output is returned as `Ok` inside the chain, the methods of [`Process`]
/// return it as an error. The output of a run replayed instead of running its
/// command, e.g. by [`DryRun`](crate::interceptor::DryRun) or a
/// [`CommandCache`](crate::CommandCache), has its success set from its status
/// like for a command that ran, its duration is not compared with the
/// [`CommandOptions::timeout`].
///
/// # Examples
///
/// ```rust
/// use agplatform::interceptor::{BoxFuture, Interceptor, Next};
/// use agplatform::{CommandExt, CommandOutput, Process, ProcessCommand, Result, TestProcess};
/// use std::process::Command;
/// use std::sync::{Arc, Mutex};
///
/// #[derive(Clone, Default)]
/// struct Failures(Arc<Mutex<Vec<String>>>);
///
/// impl Interceptor<ProcessCommand> for Failures {
///     fn handle<'a>(
///         &'a self,
///         mut next: Next<'a, ProcessCommand>,
///     ) -> BoxFuture<'a, Result<CommandOutput>> {
///         Box::pin(async move {
///             let result = next.attempt().await;
///
///             if result.as_ref().is_ok_and(|output| !output.success) {
///                 self.0.lock().unwrap().push(next.run().subject().command_line());
///             }
///
///             result
///         })
///     }
/// }
///
/// let failures = Failures::default();
/// let process = TestProcess::new();
/// process.add_interceptor(failures.clone());
/// process.set_output("git", CommandOutput { status: Some(1), ..Default::default() });
///
/// let _ = Command::new("git").arg("push").run_blocking(&process);
///
/// assert_eq!(*failures.0.lock().unwrap(), ["git push"]);
/// ```
pub struct ProcessCommand {
    pub(super) program: OsString,
    args: Vec<OsString>,
    envs: Vec<(OsString, Option<OsString>)>,
    dir: Option<PathBuf>,
    options: CommandOptions,
    current_dir: Option<PathBuf>,
    /// The variables of the env the command was resolved in as its run
    /// started, `None` while they are those of the current process, none
    /// before.
    vars: Option<Vec<(OsString, OsString)>>,
}

/// Options for [`Process::run_blocking`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandOptions {
    /// The number of the last lines of the standard output and error shown
    /// when displaying the [`CommandOutput`], e.g. as the source of an
    /// error, at most as many KiB of each. 10 by default.
    pub display_lines: usize,

    /// Passes only the environment variables set by the command to the
    /// child process, without the configured ones.
    pub clear_env: bool,

    /// Discards the standard output and error of the child process instead
    /// of capturing (and streaming) them, e.g. when they are large or hold
    /// secrets. The [`CommandOutput`] then has only the status and the
    /// duration.
    pub silent: bool,

    /// Writes the bytes to the standard input of the child process and
    /// closes it, instead of a null standard input.
    pub input: Option<Vec<u8>>,

    /// Connects the child process to the standard input, output and error
    /// of the current process, e.g. for prompts or editors. Its output is
    /// therefore not captured and [`CommandOptions::stream`] has no effect.
    ///
    /// Cannot be combined with [`CommandOptions::input`] or
    /// [`CommandOptions::timeout`], as a process group on Unix cannot read from
    /// the terminal, nor with [`CommandOptions::silent`], as it would hide
    /// the prompts.
    pub interactive: bool,

    /// Writes the output of the child process to the standard output and error
    /// of the current process as it arrives, in addition to capturing it, by
    /// [`StdStream`](crate::StdStream). Has no effect when the interceptors of
    /// the run do not include it, e.g. after [`Process::clear_interceptors`] or
    /// with its own ones built from [`Interceptors::new`].
    pub stream: bool,

    /// Kills the child process with its process group on Unix, or its job
    /// on Windows, when it does not finish in time. Descendants that leave
    /// them, e.g. daemons calling `setsid`, keep running.
    ///
    /// On Unix the child process becomes the leader of a new process
    /// group, so it does not receive the signals of the terminal, e.g.
    /// Ctrl-C. On Linux it is therefore killed when the thread that started
    /// it exits, e.g. when the current process is killed. That covers only
    /// the child process, its descendants in the group survive. A
    /// synchronous run blocks that thread until it ends, so the child
    /// outlives the thread only when the current process dies. macOS and the
    /// other Unix systems have no equivalent. On Windows the job of every
    /// child process is killed when the current process dies.
    ///
    /// A [`Duration`] too long for a deadline, e.g. [`Duration::MAX`], does
    /// not limit the run.
    pub timeout: Option<Duration>,
}

impl ProcessCommand {
    /// Creates the command of `command` with the default
    /// [`CommandOptions`].
    pub fn new(command: &Command) -> Self {
        Self::with_options(command, CommandOptions::default())
    }

    /// Creates the command of `command` with `options`, e.g. read from a
    /// configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandOptions, ProcessCommand};
    /// use std::process::Command;
    ///
    /// let options = CommandOptions { stream: true, ..Default::default() };
    /// let command = ProcessCommand::with_options(&Command::new("git"), options);
    ///
    /// assert!(command.options().stream);
    /// ```
    pub fn with_options(command: &Command, options: CommandOptions) -> Self {
        Self {
            program: command.get_program().to_owned(),
            args: command.get_args().map(OsStr::to_owned).collect(),
            envs: command
                .get_envs()
                .map(|(key, value)| (key.to_owned(), value.map(OsStr::to_owned)))
                .collect(),
            dir: command.get_current_dir().map(Path::to_path_buf),
            options,
            current_dir: None,
            vars: Some(Vec::new()),
        }
    }

    /// Returns the arguments.
    pub fn args(&self) -> &[OsString] {
        &self.args
    }

    /// Returns the arguments to be changed, e.g. by an interceptor.
    pub fn args_mut(&mut self) -> &mut Vec<OsString> {
        &mut self.args
    }

    /// Returns the key of the command for a
    /// [`Cache`](crate::interceptor::Cache): the directory, the program and the
    /// arguments, the environment variables set or removed by the command and
    /// the platform variables `vars` (unless [`CommandOptions::clear_env`]),
    /// the options changing the output and the input, each quoted or escaped so
    /// that different commands have different keys. `None` for an
    /// [`CommandOptions::interactive`] command, whose output is not captured.
    ///
    /// The variables are named with whether they are set, their values are
    /// only hashed together (FNV-1a, the same on every Rust version), as
    /// they may hold secrets, e.g. tokens, that the persisted
    /// [`entries`](crate::interceptor::Cache::entries) must not show.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::interceptor::{Cache, Run};
    /// use agplatform::{CommandExt, CommandOutput, Process, ProcessCommand, TestProcess};
    /// use std::process::Command;
    ///
    /// let cache = Cache::new(|run: &Run<ProcessCommand>| {
    ///     let command = run.subject();
    ///     (command.program() == "git").then(|| command.cache_key(&[])).flatten()
    /// });
    /// let process = TestProcess::new();
    /// process.add_interceptor(cache.clone());
    /// process.set_output("", CommandOutput::default());
    ///
    /// Command::new("git").arg("log").run_blocking(&process).unwrap();
    /// Command::new("date").run_blocking(&process).unwrap();
    ///
    /// assert_eq!(cache.entries().len(), 1);
    /// ```
    pub fn cache_key(&self, vars: &[OsString]) -> Option<String> {
        let options = self.options();

        if options.interactive {
            return None;
        }

        let vars: Vec<(&OsStr, Option<OsString>)> = if options.clear_env {
            Vec::new()
        } else {
            vars.iter()
                .map(|name| (name.as_os_str(), self.var(name)))
                .collect()
        };
        fn names<'a>(
            vars: impl Iterator<Item = (&'a OsStr, bool)>,
            unset: &'static str,
        ) -> Vec<(&'a OsStr, &'static str)> {
            vars.map(|(name, set)| (name, if set { "set" } else { unset }))
                .collect()
        }

        let mut envs: Vec<&(OsString, Option<OsString>)> = self.envs().iter().collect();
        envs.sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let values = crate::utils::fnv1a(
            envs.iter()
                .map(|(_, value)| value.as_deref())
                .chain(vars.iter().map(|(_, value)| value.as_deref()))
                .map(|value| value.map(OsStr::as_encoded_bytes)),
        );
        let mut key = format!(
            "dir: {:?}\ncommand: {:?}\nenvs: {:?}\nclear_env: {}\nsilent: {}",
            self.current_dir().unwrap_or(Path::new("")),
            std::iter::once(&self.program)
                .chain(&self.args)
                .collect::<Vec<_>>(),
            names(
                envs.iter()
                    .map(|(name, value)| (name.as_os_str(), value.is_some())),
                "removed"
            ),
            options.clear_env,
            options.silent,
        );

        if !options.clear_env {
            key.push_str(&format!(
                "\nvars: {:?}",
                names(
                    vars.iter().map(|(name, value)| (*name, value.is_some())),
                    "unset"
                )
            ));
        }

        key.push_str(&format!("\nvalues: {values:016x}"));

        if let Some(input) = &options.input {
            key.push_str(&format!("\ninput: {}", input.escape_ascii()));
        }

        Some(key)
    }

    /// Returns the program and the arguments separated by spaces.
    pub fn command_line(&self) -> String {
        std::iter::once(&self.program)
            .chain(&self.args)
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Returns the directory the command runs in, i.e. its own resolved
    /// against the current directory of the process, once its run started,
    /// e.g. when passed to an interceptor. `None` before.
    pub fn current_dir(&self) -> Option<&Path> {
        self.current_dir.as_deref()
    }

    /// Returns the environment variables set (`Some`) or removed (`None`) by
    /// the command, in the order they were set.
    pub fn envs(&self) -> &[(OsString, Option<OsString>)] {
        &self.envs
    }

    /// Returns the options.
    pub fn options(&self) -> &CommandOptions {
        &self.options
    }

    /// Returns the options to be changed, e.g. by an interceptor.
    pub fn options_mut(&mut self) -> &mut CommandOptions {
        &mut self.options
    }

    /// Returns the program, lossily converted to UTF-8.
    pub fn program(&self) -> Cow<'_, str> {
        self.program.to_string_lossy()
    }

    /// Removes the environment variable `key` for the command, e.g. in an
    /// interceptor.
    pub fn remove_env<K: AsRef<OsStr>>(&mut self, key: K) {
        self.change_env(key.as_ref(), None);
    }

    /// Returns the [`Retry`] of the failed attempts of the runs [safe to
    /// retry](Self::safe_to_retry), up to `retries` times after `delay`.
    /// Its [`when`](Retry::when) decides instead of the safety, e.g. to
    /// retry also a timeout. Each attempt is limited by the
    /// [`CommandOptions::timeout`] on its own.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandExt, CommandOutput, Process, ProcessCommand, TestProcess};
    /// use std::process::Command;
    /// use std::time::Duration;
    ///
    /// let process = TestProcess::new();
    /// process.add_interceptor(ProcessCommand::retry(3, Duration::from_secs(5)).when(|run, result| {
    ///     run.subject().safe_to_retry(result)
    ///         && result.as_ref().is_ok_and(|output| output.stderr_lossy().contains("Could not resolve host"))
    /// }));
    /// process.set_output("git push", CommandOutput { status: Some(1), ..Default::default() });
    /// process.set_io_error("svn", std::io::ErrorKind::NotFound);
    ///
    /// assert!(Command::new("git").arg("push").run_blocking(&process).is_err());
    /// assert!(Command::new("svn").run_blocking(&process).is_err());
    /// assert_eq!(process.calls(), ["git push", "svn"]);
    /// ```
    pub fn retry(retries: u32, delay: Duration) -> Retry<ProcessCommand> {
        Retry::<Self>::new(retries, delay).when(|run, result| run.subject().safe_to_retry(result))
    }

    /// Returns `true` if the failed `result` of an attempt of the command
    /// may pass when retried: an output that is not a success, e.g. of a
    /// non-zero exit code or a signal, unless it timed out, which would
    /// take as long again. Not an error, e.g. of a program not found, which
    /// would fail the same way. Used by [`Self::retry`].
    pub fn safe_to_retry(&self, result: &Result<CommandOutput>) -> bool {
        let _ = self;
        result
            .as_ref()
            .is_ok_and(|output| !output.success && !output.timed_out)
    }

    /// Sets the environment variable `key` to `value` for the command, e.g.
    /// in an interceptor.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::interceptor::{BoxFuture, Interceptor, Next};
    /// use agplatform::{CommandOutput, ProcessCommand, Result};
    ///
    /// struct Token;
    ///
    /// impl Interceptor<ProcessCommand> for Token {
    ///     fn handle<'a>(
    ///         &'a self,
    ///         mut next: Next<'a, ProcessCommand>,
    ///     ) -> BoxFuture<'a, Result<CommandOutput>> {
    ///         next.run_mut().subject_mut().set_env("TOKEN", "secret");
    ///         next.forward()
    ///     }
    /// }
    /// ```
    pub fn set_env<K: AsRef<OsStr>, V: AsRef<OsStr>>(&mut self, key: K, value: V) {
        self.change_env(key.as_ref(), Some(value.as_ref().to_owned()));
    }

    /// Sets the program, e.g. in an interceptor.
    pub fn set_program<T: Into<OsString>>(&mut self, program: T) {
        self.program = program.into();
    }

    /// [internal]
    ///
    /// Resolves the directory of the command against the current directory
    /// of `env` and takes its variables, as the run starts.
    ///
    /// WHY: taken once so that all attempts of the run get the same env,
    /// even when it is changed in between.
    pub(crate) fn resolve(&mut self, env: &EnvImpl) {
        let current_dir = env.current_dir_arc();
        self.current_dir = Some(
            self.dir
                .as_ref()
                .map_or(current_dir.to_path_buf(), |dir| current_dir.join(dir)),
        );
        self.vars = env.changed_vars();
    }

    /// [internal]
    ///
    /// Returns the variables of the env the command was resolved in, `None`
    /// while they are those of the current process.
    pub(crate) fn vars(&self) -> Option<&[(OsString, OsString)]> {
        self.vars.as_deref()
    }

    /// [internal]
    ///
    /// Returns the variable `key` of the env the command was resolved in.
    pub(crate) fn var(&self, key: &OsStr) -> Option<OsString> {
        match &self.vars {
            Some(vars) => vars
                .iter()
                .find(|(k, _)| crate::env::is_same_key(k, key))
                .map(|(_, value)| value.clone()),
            None => std::env::var_os(key),
        }
    }

    /// Records the change of the variable `key` to `value`, replacing an
    /// earlier one.
    ///
    /// WHY: the keys are compared like the platform does, case-insensitively
    /// on Windows.
    fn change_env(&mut self, key: &OsStr, value: Option<OsString>) {
        #[cfg(windows)]
        self.envs.retain(|(k, _)| !k.eq_ignore_ascii_case(key));
        #[cfg(not(windows))]
        self.envs.retain(|(k, _)| k != key);
        self.envs.push((key.to_owned(), value));
    }
}

impl Subject for ProcessCommand {
    type Output = CommandOutput;
    type Stream = OutputStream;

    /// Rejects the combinations of the options that cannot work, also with
    /// a sink, see [`Process::run_blocking`] and [`Process::run_blocking_with_sink`].
    fn validate(run: &Run<Self>) -> Result {
        validate(
            &run.subject().program(),
            run.subject().options(),
            run.has_sink(),
        )
    }

    /// Passes the standard output, then the standard error of `output` on,
    /// each in one chunk unless empty, none when the run is
    /// [`CommandOptions::silent`] or [`CommandOptions::interactive`], which
    /// capture no output.
    fn replay<'a>(
        run: &'a Run<Self>,
        output: &'a CommandOutput,
        emit: &'a mut Emit<'_, Self>,
    ) -> BoxFuture<'a, Result> {
        Box::pin(async move {
            let options = run.subject().options();

            if !options.silent && !options.interactive {
                for (stream, data) in [
                    (OutputStream::Stdout, &output.stdout),
                    (OutputStream::Stderr, &output.stderr),
                ] {
                    if !data.is_empty() {
                        emit.emit(stream, data).await?;
                    }
                }
            }

            Ok(())
        })
    }

    /// Returns `output` successful if it exited with code `0` in time, with
    /// the standard output and error `captured` and the
    /// [`CommandOptions::display_lines`] of the run.
    fn finish(
        run: &Run<Self>,
        output: CommandOutput,
        mut captured: Captured<Self>,
    ) -> CommandOutput {
        CommandOutput {
            success: output.status == Some(0) && !output.timed_out,
            stdout: captured.take(OutputStream::Stdout),
            stderr: captured.take(OutputStream::Stderr),
            display_lines: run.subject().options().display_lines,
            ..output
        }
    }
}

impl Run<ProcessCommand> {
    /// Passes only the environment variables set by the command.
    ///
    /// See [`CommandOptions::clear_env`].
    pub fn clear_env(mut self) -> Self {
        self.subject_mut().options.clear_env = true;
        self
    }

    /// Shows the last `lines` of the output when displayed.
    ///
    /// See [`CommandOptions::display_lines`].
    pub fn display_lines(mut self, lines: usize) -> Self {
        self.subject_mut().options.display_lines = lines;
        self
    }

    /// Discards the output of the command.
    ///
    /// See [`CommandOptions::silent`].
    pub fn silent(mut self) -> Self {
        self.subject_mut().options.silent = true;
        self
    }

    /// Writes `input` to the standard input of the command.
    ///
    /// See [`CommandOptions::input`].
    pub fn input<T: Into<Vec<u8>>>(mut self, input: T) -> Self {
        self.subject_mut().options.input = Some(input.into());
        self
    }

    /// Connects the command to the terminal.
    ///
    /// See [`CommandOptions::interactive`].
    pub fn interactive(mut self) -> Self {
        self.subject_mut().options.interactive = true;
        self
    }

    /// Streams the output of the command.
    ///
    /// See [`CommandOptions::stream`].
    pub fn stream(mut self) -> Self {
        self.subject_mut().options.stream = true;
        self
    }

    /// Limits the command to `timeout`.
    ///
    /// See [`CommandOptions::timeout`].
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.subject_mut().options.timeout = Some(timeout);
        self
    }

    /// Runs the command with its own `interceptors` instead of those of the
    /// process, see [`Process::interceptors`]. They are taken once as the
    /// run starts, so the interceptors cannot change them.
    pub fn interceptors(mut self, interceptors: Interceptors<ProcessCommand>) -> Self {
        self.insert(OwnInterceptors(interceptors));
        self
    }

    /// Runs the command with the options.
    ///
    /// See [`Process::run_blocking`].
    pub fn run_blocking<P: Process + ?Sized>(self, process: &P) -> Result<CommandOutput> {
        process.run_blocking(self)
    }

    /// Runs the command with the options, passing its output to `sink`.
    ///
    /// See [`Process::run_blocking_with_sink`].
    pub fn run_blocking_with_sink<P: Process + ?Sized>(
        self,
        process: &P,
        sink: &mut dyn Sink<ProcessCommand>,
    ) -> Result<CommandOutput> {
        process.run_blocking_with_sink(self, sink)
    }

    /// Enabled by the `tokio` feature flag.
    ///
    /// Runs the command asynchronously with the options.
    ///
    /// See [`Process::run_async`].
    #[cfg(feature = "tokio")]
    pub fn run_async<P: Process + ?Sized>(
        self,
        process: &P,
    ) -> impl Future<Output = Result<CommandOutput>> + Send {
        process.run_async(self)
    }

    /// Enabled by the `tokio` feature flag.
    ///
    /// Runs the command asynchronously with the options, passing its output
    /// to `sink`.
    ///
    /// See [`Process::run_async_with_sink`].
    #[cfg(feature = "tokio")]
    pub fn run_async_with_sink<P: Process + ?Sized>(
        self,
        process: &P,
        sink: &mut dyn Sink<ProcessCommand>,
    ) -> impl Future<Output = Result<CommandOutput>> + Send {
        process.run_async_with_sink(self, sink)
    }
}

impl From<&Command> for Run<ProcessCommand> {
    fn from(command: &Command) -> Self {
        Self::new(ProcessCommand::new(command))
    }
}

impl From<&mut Command> for Run<ProcessCommand> {
    fn from(command: &mut Command) -> Self {
        Self::new(ProcessCommand::new(command))
    }
}

impl From<ProcessCommand> for Run<ProcessCommand> {
    fn from(command: ProcessCommand) -> Self {
        Self::new(command)
    }
}

impl Default for CommandOptions {
    fn default() -> Self {
        Self {
            display_lines: DISPLAY_LINES,
            clear_env: false,
            input: None,
            interactive: false,
            silent: false,
            stream: false,
            timeout: None,
        }
    }
}

/// Rejects the combinations of `options` that cannot work, also with a
/// `sink`.
fn validate(program: &str, options: &CommandOptions, sink: bool) -> Result<()> {
    if options.interactive && options.input.is_some() {
        Err(process_error!(
            "'{program}': interactive cannot be combined with input"
        ))
    } else if options.interactive && options.silent {
        Err(process_error!(
            "'{program}': interactive cannot be combined with silent"
        ))
    } else if options.interactive && options.timeout.is_some() {
        Err(process_error!(
            "'{program}': interactive cannot be combined with timeout"
        ))
    } else if sink && options.interactive {
        Err(process_error!(
            "'{program}': interactive cannot be combined with a sink"
        ))
    } else if sink && options.silent {
        Err(process_error!(
            "'{program}': silent cannot be combined with a sink"
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestDir;
    use crate::interceptor::Chunk;
    use crate::interceptor::Interceptor;
    use crate::process::CommandExt;
    use crate::process::ProcessImpl;
    use crate::process::process_impl::StdExecutor;
    use crate::process::tests::*;

    mod process_command {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn new_should_use_default_options() {
                let command = ProcessCommand::new(&Command::new("git"));

                assert_eq!(*command.options(), CommandOptions::default());
            }
        }

        mod with_options {
            use super::*;

            #[test]
            fn with_options_should_run_command_with_options() {
                let (process, recorder) = observed();
                let options = CommandOptions {
                    display_lines: 3,
                    ..Default::default()
                };

                process
                    .run_blocking(with_options(&shell("exit 0"), options.clone()))
                    .unwrap();

                assert_eq!(recorder.options(), options);
            }
        }

        mod args {
            use super::*;

            #[test]
            fn args_should_return_args_of_command() {
                let mut command = Command::new("git");
                command.args(["commit", "-m", "message"]);

                assert_eq!(
                    ProcessCommand::new(&command).args(),
                    ["commit", "-m", "message"]
                );
            }
        }

        mod args_mut {
            use super::*;

            #[test]
            fn args_mut_should_change_args_of_command() {
                let mut command = ProcessCommand::new(&Command::new("git"));

                command.args_mut().push("status".into());

                assert_eq!(command.args(), ["status"]);
            }
        }

        mod cache_key {
            use super::*;

            /// Returns the command `printf` with `args` in the env with
            /// `vars`.
            fn printf(args: &[&str], vars: &[(&str, &str)]) -> ProcessCommand {
                let mut command = ProcessCommand::new(Command::new("printf").args(args));
                command.vars = Some(
                    vars.iter()
                        .map(|(key, value)| (key.into(), value.into()))
                        .collect(),
                );
                command
            }

            #[test]
            fn cache_key_should_name_dir_command_vars_and_input() {
                let mut command = Command::new("git");
                command.arg("status").env("SET", "1").env_remove("REMOVED");
                let options = CommandOptions {
                    input: Some(b"in".to_vec()),
                    ..Default::default()
                };

                assert_eq!(
                    with_options(&command, options).cache_key(&["UNSET".into()]),
                    Some(format!(
                        "dir: \"\"\ncommand: [\"git\", \"status\"]\n\
                         envs: [(\"REMOVED\", \"removed\"), (\"SET\", \"set\")]\n\
                         clear_env: false\nsilent: false\nvars: [(\"UNSET\", \"unset\")]\n\
                         values: {:016x}\ninput: in",
                        crate::utils::fnv1a([None, Some(b"1".as_slice()), None])
                    ))
                );
            }

            #[test]
            fn cache_key_envs_changed_in_other_order_should_be_equal() {
                let changed = |changes: &[(&str, Option<&str>)]| {
                    let mut command = printf(&[], &[]);
                    for &(key, value) in changes {
                        match value {
                            Some(value) => command.set_env(key, value),
                            None => command.remove_env(key),
                        }
                    }
                    command.cache_key(&[])
                };

                assert_eq!(
                    changed(&[("A", Some("1")), ("B", Some("2")), ("C", None)]),
                    changed(&[("C", None), ("B", Some("2")), ("A", Some("1"))])
                );
            }

            #[test]
            fn cache_key_should_ignore_platform_vars_not_named() {
                let without = printf(&[], &[]).cache_key(&["NAMED".into()]);

                let with = printf(&[], &[("CI_JOB_ID", "42")]).cache_key(&["NAMED".into()]);

                assert_eq!(with, without);
            }

            #[test]
            fn cache_key_should_hash_values_of_vars() {
                let mut command = printf(&[], &[("PLATFORM_TOKEN", "t0k3n")]);
                command.set_env("TOKEN", "s3cr3t");

                let key = command.cache_key(&["PLATFORM_TOKEN".into()]).unwrap();

                assert!(!key.contains("s3cr3t") && !key.contains("t0k3n"), "{key}");
                assert!(
                    key.contains("vars: [(\"PLATFORM_TOKEN\", \"set\")]"),
                    "{key}"
                );
            }

            #[test]
            fn cache_key_should_differ_for_commands_with_different_output() {
                let changed = |change: fn(&mut CommandOptions)| {
                    let mut command = printf(&["a"], &[]);
                    change(command.options_mut());
                    command
                };
                let commands = [
                    printf(&["a b"], &[]),
                    printf(&["a", "b"], &[]),
                    printf(&["a"], &[]),
                    changed(|options| options.silent = true),
                    changed(|options| options.clear_env = true),
                    printf(&["a"], &[("VAR", "1")]),
                    printf(&["a"], &[("VAR", "2")]),
                    changed(|options| options.input = Some(b"\xFF".to_vec())),
                    changed(|options| options.input = Some(b"\xFE".to_vec())),
                ];

                let keys = commands
                    .iter()
                    .map(|command| command.cache_key(&["VAR".into()]))
                    .collect::<std::collections::BTreeSet<_>>();

                assert_eq!(keys.len(), commands.len(), "{keys:?}");
            }

            #[test]
            fn cache_key_interactive_should_be_none() {
                let mut command = printf(&[], &[]);
                command.options_mut().interactive = true;

                assert_eq!(command.cache_key(&[]), None);
            }
        }

        mod command_line {
            use super::*;

            #[test]
            fn command_line_should_join_program_and_args() {
                let mut command = Command::new("git");
                command.args(["commit", "-m", "message"]);

                assert_eq!(
                    ProcessCommand::new(&command).command_line(),
                    "git commit -m message"
                );
            }
        }

        mod current_dir {
            use super::*;

            #[test]
            fn current_dir_should_be_resolved_dir_of_command_when_run() {
                let (process, recorder) = observed();
                let mut command = shell("exit 0");
                command.current_dir("sub");

                let _ = process.run_blocking(&command);

                assert_eq!(ProcessCommand::new(&command).current_dir(), None);
                assert_eq!(
                    recorder.events()[0],
                    format!("started in {}", std::env::temp_dir().join("sub").display())
                );
            }
        }

        mod envs {
            use super::*;

            #[test]
            fn envs_should_return_vars_set_and_removed_by_command() {
                let mut command = Command::new("git");
                command.env("SET", "1").env_remove("REMOVED");

                assert_eq!(
                    ProcessCommand::new(&command).envs(),
                    [
                        (OsString::from("REMOVED"), None),
                        (OsString::from("SET"), Some(OsString::from("1")))
                    ]
                );
            }
        }

        mod options_mut {
            use super::*;

            #[test]
            fn options_mut_should_change_options_of_command() {
                let mut command = ProcessCommand::new(&Command::new("git"));

                command.options_mut().silent = true;

                assert!(command.options().silent);
            }
        }

        mod program {
            use super::*;

            #[test]
            fn program_should_return_program_without_args() {
                let mut command = Command::new("git");
                command.args(["clone", "https://user:token@example.com/repo"]);

                assert_eq!(ProcessCommand::new(&command).program(), "git");
            }
        }

        mod remove_env {
            use super::*;

            #[test]
            fn remove_env_should_remove_var_for_command() {
                let process = process(std::env::temp_dir());
                process.add_interceptor(Inspect(|run: &mut Run<ProcessCommand>| {
                    run.subject_mut().remove_env("AGPLATFORM_REMOVED");
                }));

                let output = process
                    .run_blocking(print_var("AGPLATFORM_REMOVED").env("AGPLATFORM_REMOVED", "1"))
                    .unwrap();

                assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "unset");
            }
        }

        mod retry {
            use super::*;

            #[test]
            fn retry_should_retry_failures_safe_to_retry_only() {
                for (command, timeout, attempts) in [
                    (shell("exit 3"), Duration::from_secs(10), 2),
                    (shell(SLEEP_THEN_ECHO), Duration::from_millis(100), 1),
                    (
                        Command::new("agplatform_missing_program"),
                        Duration::from_secs(10),
                        1,
                    ),
                ] {
                    let (process, recorder) = observed();
                    let program = command.get_program().to_owned();

                    let _ = Run::from(&command)
                        .timeout(timeout)
                        .interceptors(retried(ProcessCommand::retry(1, Duration::ZERO), &recorder))
                        .run_blocking(&process);

                    assert_eq!(
                        recorder.events().len(),
                        attempts * 2,
                        "program: {program:?}"
                    );
                }
            }
        }

        mod safe_to_retry {
            use super::*;

            #[test]
            fn safe_to_retry_should_accept_failed_outputs_that_did_not_time_out() {
                let command = ProcessCommand::new(&Command::new("git"));

                for (result, safe) in [
                    (Ok(exit(1)), true),
                    (
                        Ok(CommandOutput {
                            status: None,
                            signal: Some(9),
                            ..Default::default()
                        }),
                        true,
                    ),
                    (
                        Ok(CommandOutput {
                            status: None,
                            timed_out: true,
                            ..Default::default()
                        }),
                        false,
                    ),
                    (
                        Ok(CommandOutput {
                            success: true,
                            ..Default::default()
                        }),
                        false,
                    ),
                    (Err(crate::Error::process("not found")), false),
                ] {
                    assert_eq!(command.safe_to_retry(&result), safe, "{result:?}");
                }
            }
        }

        mod set_env {
            use super::*;

            #[test]
            fn set_env_should_replace_var_of_command() {
                let mut command =
                    ProcessCommand::new(Command::new("git").env("A", "1").env("B", "1"));

                command.set_env("A", "2");

                assert_eq!(
                    command.envs(),
                    [
                        (OsString::from("B"), Some(OsString::from("1"))),
                        (OsString::from("A"), Some(OsString::from("2")))
                    ]
                );
            }

            #[test]
            fn set_env_should_pass_var_to_child() {
                let process = process(std::env::temp_dir());
                process.add_interceptor(Inspect(|run: &mut Run<ProcessCommand>| {
                    run.subject_mut().set_env("AGPLATFORM_CHANGED", "changed");
                }));

                let output = process
                    .run_blocking(&print_var("AGPLATFORM_CHANGED"))
                    .unwrap();

                assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "changed");
            }
        }

        mod set_program {
            use super::*;

            #[test]
            fn set_program_should_change_program_of_command() {
                let mut command = ProcessCommand::new(&Command::new("git"));

                command.set_program("hg");

                assert_eq!(command.program(), "hg");
            }
        }

        mod var {
            use super::*;

            #[test]
            fn var_should_return_var_of_env_command_was_resolved_in() {
                let env = EnvImpl::new(vec![("KEY".into(), "value".into())], PathBuf::new());
                let mut command = ProcessCommand::new(&shell("exit 0"));

                command.resolve(&env);

                assert_eq!(command.var(OsStr::new("KEY")), Some("value".into()));
                assert_eq!(command.var(OsStr::new("MISSING")), None);
            }

            #[test]
            fn var_unchanged_vars_of_process_should_return_var_of_process() {
                let mut command = ProcessCommand::new(&shell("exit 0"));

                command.resolve(&EnvImpl::new_from_std());

                assert_eq!(command.vars(), None);
                assert_eq!(
                    command.var(OsStr::new(INHERITED_VAR)),
                    std::env::var_os(INHERITED_VAR)
                );
            }
        }

        mod subject {
            use super::*;

            /// Returns the output of `run` by `process` through no
            /// interceptors, failed or not.
            fn ran(process: &ProcessImpl, mut run: Run<ProcessCommand>) -> Result<CommandOutput> {
                run.subject_mut().resolve(&process.env);
                Interceptors::new().run_blocking(&StdExecutor(process), &mut run, None)
            }

            mod validate {
                use super::*;

                #[test]
                fn validate_should_reject_options_not_combining_with_sink() {
                    let process = process(std::env::temp_dir());
                    let mut sunk = (Vec::new(), Vec::new());

                    for (run, option) in [
                        (shell("exit 0").silent(), "silent"),
                        (shell("exit 0").interactive(), "interactive"),
                    ] {
                        let error = process
                            .run_blocking_with_sink(run, &mut sink_into(&mut sunk))
                            .unwrap_err();

                        assert!(
                            error
                                .description()
                                .ends_with(&format!("': {option} cannot be combined with a sink")),
                            "{error}"
                        );
                    }
                }

                #[test]
                fn validate_options_made_invalid_by_interceptor_should_fail_attempt() {
                    let (process, recorder) = observed();
                    process.add_interceptor(Inspect(|run: &mut Run<ProcessCommand>| {
                        run.subject_mut().options_mut().interactive = true;
                        run.subject_mut().options_mut().silent = true;
                    }));

                    let error = process.run_blocking(&shell("exit 0")).unwrap_err();

                    assert!(
                        error
                            .description()
                            .ends_with("': interactive cannot be combined with silent")
                    );
                    assert_eq!(recorder.events()[1], format!("finished {error}"));
                }
            }

            mod replay {
                use super::*;

                #[test]
                fn replay_finished_output_should_finish_to_same_output() {
                    let output = CommandOutput {
                        stdout: b"out".to_vec(),
                        stderr: b"err".to_vec(),
                        ..exit(1)
                    };

                    for run in [
                        Run::from(&shell("exit 0")),
                        shell("exit 0").display_lines(3),
                        shell("exit 0").silent(),
                        shell("exit 0").interactive(),
                    ] {
                        crate::interceptor::tests::subject::replay::replay_finished_output_should_finish_to_same_output(
                            &run,
                            &output,
                        );
                    }
                }

                #[test]
                fn replay_should_pass_output_on_as_if_it_ran() {
                    // GIVEN an interceptor replaying an output of a streamed run
                    let dir = TestDir::new();
                    let (process, recorder) = observed();
                    process.set_current_dir(dir.path().to_path_buf());
                    process.add_interceptor(Replay(CommandOutput {
                        stdout: b"set".to_vec(),
                        stderr: b"err".to_vec(),
                        ..exit(1)
                    }));

                    // WHEN running a command that would create a marker
                    let error = shell("touch marker")
                        .stream()
                        .run_blocking(&process)
                        .unwrap_err();

                    // THEN the output is returned and passed on as if it ran
                    let output = error.command_output().unwrap();
                    assert_eq!(
                        (&*output.stdout, &*output.stderr),
                        (&b"set"[..], &b"err"[..])
                    );
                    assert_eq!(
                        (recorder.stdout(), recorder.stderr()),
                        (b"set".to_vec(), b"err".to_vec())
                    );
                    assert!(!dir.path().join("marker").exists());
                }

                #[test]
                fn replay_silent_or_interactive_should_pass_nothing_on() {
                    let (process, recorder) = observed();
                    process.add_interceptor(Replay(CommandOutput {
                        stdout: b"out".to_vec(),
                        ..exit(0)
                    }));

                    for run in [shell("exit 0").silent(), shell("exit 0").interactive()] {
                        let output = process.run_blocking(run).unwrap();

                        assert!(output.stdout.is_empty());
                    }

                    assert!(recorder.stdout().is_empty());
                }

                #[test]
                fn replay_failing_to_pass_on_should_stop_with_error() {
                    let process = process(std::env::temp_dir());
                    process.add_interceptor(Replay(CommandOutput {
                        stdout: b"out".to_vec(),
                        stderr: b"err".to_vec(),
                        ..exit(0)
                    }));
                    let mut streams = Vec::new();

                    let error = process
                        .run_blocking_with_sink(
                            &shell("exit 0"),
                            &mut |stream: OutputStream, _: &[u8]| {
                                streams.push(stream);
                                Err(crate::Error::process("full"))
                            },
                        )
                        .unwrap_err();

                    assert_eq!(error.description(), "full");
                    assert_eq!(streams, [OutputStream::Stdout]);
                }
            }

            mod finish {
                use super::*;

                #[test]
                fn finish_should_set_success_from_status_and_timeout() {
                    let process = process(std::env::temp_dir());

                    for (status, timed_out, success) in [
                        (Some(0), false, true),
                        (Some(1), false, false),
                        (None, false, false),
                        (Some(0), true, false),
                    ] {
                        let output = CommandOutput {
                            success: !success,
                            status,
                            timed_out,
                            ..Default::default()
                        };
                        let run = Run::from(&shell("exit 0")).with_replay_output(output);

                        let finished = ran(&process, run).unwrap();

                        assert_eq!(finished.success, success, "{status:?} {timed_out}");
                    }
                }

                #[test]
                fn finish_should_take_display_lines_of_run() {
                    let process = process(std::env::temp_dir());

                    let output = ran(&process, shell("exit 0").display_lines(3)).unwrap();

                    assert_eq!(output.display_lines, 3);
                }

                #[test]
                fn finish_should_take_output_as_passed_on() {
                    // GIVEN an interceptor redacting the chunks
                    struct Redact;

                    impl Interceptor<ProcessCommand> for Redact {
                        fn chunk(&self, mut chunk: Chunk<ProcessCommand>) -> Result {
                            chunk.set_data(b"***".to_vec());
                            chunk.forward()
                        }
                    }

                    let process = process(std::env::temp_dir());
                    process.add_interceptor(Redact);

                    // WHEN running a command printing a secret
                    let output = process.run_blocking(&shell("echo secret")).unwrap();

                    // THEN the output has the redacted data
                    assert_eq!(output.stdout, b"***");
                }
            }
        }
    }

    mod run {
        use super::*;

        #[test]
        fn run_should_combine_options() {
            let (process, recorder) = observed();

            let _ = shell("exit 0")
                .timeout(Duration::from_secs(10))
                .stream()
                .clear_env()
                .silent()
                .display_lines(3)
                .input("in")
                .interactive()
                .run_blocking(&process);

            assert_eq!(
                recorder.options(),
                CommandOptions {
                    display_lines: 3,
                    clear_env: true,
                    silent: true,
                    input: Some(b"in".to_vec()),
                    interactive: true,
                    stream: true,
                    timeout: Some(Duration::from_secs(10))
                }
            );
        }

        mod clear_env {
            use super::*;

            #[test]
            fn clear_env_should_set_option() {
                let run = Run::from(&Command::new("git")).clear_env();

                assert!(run.subject().options().clear_env);
            }
        }

        mod display_lines {
            use super::*;

            #[test]
            fn display_lines_should_set_option() {
                let run = Run::from(&Command::new("git")).display_lines(3);

                assert_eq!(run.subject().options().display_lines, 3);
            }
        }

        mod silent {
            use super::*;

            #[test]
            fn silent_should_set_option() {
                let run = Run::from(&Command::new("git")).silent();

                assert!(run.subject().options().silent);
            }
        }

        mod input {
            use super::*;

            #[test]
            fn input_should_set_option() {
                let run = Run::from(&Command::new("git")).input("in");

                assert_eq!(
                    run.subject().options().input.as_deref(),
                    Some(b"in".as_slice())
                );
            }
        }

        mod interactive {
            use super::*;

            #[test]
            fn interactive_should_set_option() {
                let run = Run::from(&Command::new("git")).interactive();

                assert!(run.subject().options().interactive);
            }
        }

        mod stream {
            use super::*;

            #[test]
            fn stream_should_set_option() {
                let run = Run::from(&Command::new("git")).stream();

                assert!(run.subject().options().stream);
            }
        }

        mod timeout {
            use super::*;

            #[test]
            fn timeout_should_set_option() {
                let run = Run::from(&Command::new("git")).timeout(Duration::from_secs(1));

                assert_eq!(
                    run.subject().options().timeout,
                    Some(Duration::from_secs(1))
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

                Run::from(&shell("exit 0"))
                    .interceptors(interceptors)
                    .run_blocking(&process)
                    .unwrap();

                assert!(recorder.events().is_empty());
                assert_eq!(own.events().len(), 2);
            }
        }

        #[allow(clippy::module_inception)]
        mod run_blocking {
            use super::*;

            #[test]
            fn run_blocking_should_run_with_options() {
                let (process, recorder) = observed();

                Run::from(&shell("exit 0"))
                    .display_lines(3)
                    .run_blocking(&process)
                    .unwrap();

                assert_eq!(recorder.options().display_lines, 3);
            }
        }

        mod run_blocking_with_sink {
            use super::*;

            #[test]
            fn run_blocking_with_sink_should_run_with_options_to_sink() {
                let (process, recorder) = observed();
                let command = shell("echo out");
                let mut sunk = (Vec::new(), Vec::new());

                command
                    .display_lines(3)
                    .run_blocking_with_sink(&process, &mut sink_into(&mut sunk))
                    .unwrap();

                assert_eq!(recorder.options().display_lines, 3);
                assert_eq!(String::from_utf8_lossy(&sunk.0).trim(), "out");
            }
        }

        #[cfg(feature = "tokio")]
        mod run_async {
            use super::*;

            #[test]
            fn run_async_should_run_with_options() {
                let (process, recorder) = observed();
                let command = shell("exit 0");

                block_on(command.display_lines(3).run_async(&process)).unwrap();

                assert_eq!(recorder.options().display_lines, 3);
            }
        }

        #[cfg(feature = "tokio")]
        mod run_async_with_sink {
            use super::*;

            #[test]
            fn run_async_with_sink_should_run_with_options_to_sink() {
                let (process, recorder) = observed();
                let command = shell("echo out");
                let mut sunk = (Vec::new(), Vec::new());

                block_on(
                    command
                        .display_lines(3)
                        .run_async_with_sink(&process, &mut sink_into(&mut sunk)),
                )
                .unwrap();

                assert_eq!(recorder.options().display_lines, 3);
                assert_eq!(String::from_utf8_lossy(&sunk.0).trim(), "out");
            }
        }
    }

    mod command_options {
        use super::*;

        mod default {
            use super::*;

            #[allow(clippy::module_inception)]
            mod default {
                use super::*;

                #[test]
                fn default_should_show_ten_lines_only() {
                    assert_eq!(
                        CommandOptions::default(),
                        CommandOptions {
                            display_lines: 10,
                            clear_env: false,
                            silent: false,
                            input: None,
                            interactive: false,
                            stream: false,
                            timeout: None,
                        }
                    );
                }
            }
        }
    }

    mod from {
        use super::*;

        #[allow(clippy::module_inception)]
        mod from {
            use super::*;

            #[test]
            fn from_command_should_run_it_with_default_options() {
                let (process, recorder) = observed();
                let mut command = shell("exit 0");

                process.run_blocking(&command).unwrap();
                process.run_blocking(&mut command).unwrap();
                process.run_blocking(ProcessCommand::new(&command)).unwrap();

                assert_eq!(recorder.options(), CommandOptions::default());
                assert_eq!(recorder.events().len(), 6);
            }
        }
    }

    mod validate {
        use super::*;

        #[test]
        fn validate_should_accept_compatible_options() {
            for options in [
                CommandOptions::default(),
                CommandOptions {
                    interactive: true,
                    stream: true,
                    clear_env: true,
                    ..Default::default()
                },
                CommandOptions {
                    input: Some(Vec::new()),
                    timeout: Some(Duration::from_secs(1)),
                    ..Default::default()
                },
            ] {
                assert!(validate("git", &options, false).is_ok(), "{options:?}");
                assert!(
                    validate("git", &options, true).is_ok() != options.interactive,
                    "{options:?}"
                );
            }
        }

        #[test]
        fn validate_interactive_with_incompatible_option_should_return_error() {
            for (options, description) in [
                (
                    CommandOptions {
                        interactive: true,
                        input: Some(Vec::new()),
                        ..Default::default()
                    },
                    "'git': interactive cannot be combined with input",
                ),
                (
                    CommandOptions {
                        interactive: true,
                        timeout: Some(Duration::from_secs(1)),
                        ..Default::default()
                    },
                    "'git': interactive cannot be combined with timeout",
                ),
                (
                    CommandOptions {
                        interactive: true,
                        silent: true,
                        ..Default::default()
                    },
                    "'git': interactive cannot be combined with silent",
                ),
            ] {
                assert_eq!(
                    validate("git", &options, false).unwrap_err().description(),
                    description
                );
            }
        }

        #[test]
        fn validate_sink_without_output_should_return_error() {
            for (options, description) in [
                (
                    CommandOptions {
                        interactive: true,
                        ..Default::default()
                    },
                    "'git': interactive cannot be combined with a sink",
                ),
                (
                    CommandOptions {
                        silent: true,
                        ..Default::default()
                    },
                    "'git': silent cannot be combined with a sink",
                ),
            ] {
                assert_eq!(
                    validate("git", &options, true).unwrap_err().description(),
                    description
                );
            }
        }
    }
}
