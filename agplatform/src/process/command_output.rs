use std::borrow::Cow;
use std::time::Duration;

use crate::Result;
use crate::interceptor::Outcome;
use crate::utils::Tail;

use super::DISPLAY_LINES;

/// The output of a finished child process.
///
/// Returned from [`Process::run_blocking`](crate::Process::run_blocking) on success and by
/// [`Error::command_output`](crate::Error::command_output) of its error
/// otherwise.
///
/// Outputs are equal when they differ at most in their
/// [`success`](Self::success), which is decided by the run, and their
/// [`display_lines`](Self::display_lines), which only affects how they are
/// displayed.
#[derive(Clone, Debug)]
pub struct CommandOutput {
    /// Whether the run succeeded. Set for each attempt to whether the child
    /// process exited with code `0` in time, then may be changed by the
    /// interceptors, e.g. to accept the exit code `1` of a command
    /// answering "no".
    pub success: bool,

    /// The exit code, `None` when the process was terminated without one,
    /// e.g. by a signal or when it timed out.
    pub status: Option<i32>,

    /// The signal that terminated the process, e.g. `9` when it was killed
    /// (`SIGKILL`) or `11` when it crashed (`SIGSEGV`). Unix only, `None`
    /// on Windows, where a terminated process has an exit code.
    pub signal: Option<i32>,

    /// The captured standard output, all of it however large, see
    /// [`CommandOptions::silent`](crate::CommandOptions::silent) to discard it.
    pub stdout: Vec<u8>,

    /// The captured standard error, all of it however large.
    pub stderr: Vec<u8>,

    /// The wall time the process took.
    pub duration: Duration,

    /// Whether the process was killed for exceeding
    /// [`CommandOptions::timeout`](crate::CommandOptions::timeout).
    pub timed_out: bool,

    /// The number of the last lines of the standard output and error shown when
    /// displayed, see
    /// [`CommandOptions::display_lines`](crate::CommandOptions::display_lines).
    /// 10 by default.
    pub display_lines: usize,
}

impl CommandOutput {
    /// Returns the output of a process exiting with code `0` with `stdout`,
    /// e.g. for a test double, see
    /// [`TestProcess::set_output`](crate::TestProcess::set_output).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::CommandOutput;
    ///
    /// let output = CommandOutput::succeeded("main\n");
    /// assert_eq!((output.success, output.status), (true, Some(0)));
    /// assert_eq!(output.stdout, b"main\n");
    /// ```
    pub fn succeeded(stdout: impl Into<Vec<u8>>) -> Self {
        Self {
            success: true,
            stdout: stdout.into(),
            ..Default::default()
        }
    }

    /// Returns the output of a process exiting with code `status` with
    /// `stderr`, e.g. for a test double.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::CommandOutput;
    ///
    /// let output = CommandOutput::failed(128, "fatal: not a git repository");
    /// assert_eq!((output.success, output.status), (false, Some(128)));
    /// assert_eq!(output.stderr, b"fatal: not a git repository");
    /// ```
    pub fn failed(status: i32, stderr: impl Into<Vec<u8>>) -> Self {
        Self {
            status: Some(status),
            stderr: stderr.into(),
            ..Default::default()
        }
    }

    /// Returns the standard output as UTF-8.
    ///
    /// # Errors
    ///
    /// Returns an error if the standard output is not valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::CommandOutput;
    ///
    /// let output = CommandOutput { stdout: b"out".to_vec(), ..Default::default() };
    /// assert_eq!(output.stdout_str().unwrap(), "out");
    /// ```
    pub fn stdout_str(&self) -> Result<&str> {
        utf8("stdout", &self.stdout)
    }

    /// Returns the standard error as UTF-8.
    ///
    /// # Errors
    ///
    /// Returns an error if the standard error is not valid UTF-8.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::CommandOutput;
    ///
    /// let output = CommandOutput { stderr: b"err".to_vec(), ..Default::default() };
    /// assert_eq!(output.stderr_str().unwrap(), "err");
    /// ```
    pub fn stderr_str(&self) -> Result<&str> {
        utf8("stderr", &self.stderr)
    }

    /// Returns the standard output as UTF-8, replacing invalid sequences
    /// with `U+FFFD`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::CommandOutput;
    ///
    /// let output = CommandOutput { stdout: b"out\xFF".to_vec(), ..Default::default() };
    /// assert_eq!(output.stdout_lossy(), "out\u{FFFD}");
    /// ```
    pub fn stdout_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.stdout)
    }

    /// Returns the standard error as UTF-8, replacing invalid sequences
    /// with `U+FFFD`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::CommandOutput;
    ///
    /// let output = CommandOutput { stderr: b"err\xFF".to_vec(), ..Default::default() };
    /// assert_eq!(output.stderr_lossy(), "err\u{FFFD}");
    /// ```
    pub fn stderr_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.stderr)
    }
}

/// The empty output of a process exiting with code `0`, e.g. replayed by
/// [`DryRun`](crate::interceptor::DryRun), whose success is set by the run
/// like for a command that ran. Not a success until then, like an output
/// an interceptor returns without passing the run on.
impl Default for CommandOutput {
    fn default() -> Self {
        Self {
            success: false,
            status: Some(0),
            signal: None,
            stdout: Vec::new(),
            stderr: Vec::new(),
            duration: Duration::ZERO,
            timed_out: false,
            display_lines: DISPLAY_LINES,
        }
    }
}

impl PartialEq for CommandOutput {
    fn eq(&self, other: &Self) -> bool {
        self.status == other.status
            && self.signal == other.signal
            && self.stdout == other.stdout
            && self.stderr == other.stderr
            && self.duration == other.duration
            && self.timed_out == other.timed_out
    }
}

impl Eq for CommandOutput {}

impl std::fmt::Display for CommandOutput {
    /// Shows the status, the duration and the last
    /// [`display_lines`](Self::display_lines) of the standard output and
    /// error.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "status: {}\nduration: {:?}\nstdout:\n{}\nstderr:\n{}",
            match (self.status, self.signal) {
                (Some(code), _) => code.to_string(),
                (None, Some(signal)) => format!("signal {signal}"),
                (None, None) => "none".to_string(),
            },
            self.duration,
            Tail {
                text: &self.stdout,
                lines: self.display_lines,
            },
            Tail {
                text: &self.stderr,
                lines: self.display_lines,
            }
        )
    }
}

impl std::error::Error for CommandOutput {}

impl Outcome for CommandOutput {
    /// Returns [`Self::success`].
    fn success(&self) -> bool {
        self.success
    }

    /// Sets [`Self::success`].
    fn set_success(&mut self, success: bool) {
        self.success = success;
    }
}

fn utf8<'a>(name: &str, bytes: &'a [u8]) -> Result<&'a str> {
    std::str::from_utf8(bytes)
        .map_err(|error| process_error!("{name} is not valid UTF-8: {error}").with_source(error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interceptor::BoxFuture;
    use crate::interceptor::Interceptor;
    use crate::interceptor::Next;
    use crate::process::Process;
    use crate::process::ProcessCommand;
    use crate::process::tests::*;

    mod command_output {
        use super::*;

        mod default {
            use super::*;

            #[allow(clippy::module_inception)]
            mod default {
                use super::*;

                #[test]
                fn default_should_be_empty_exit_with_code_0_not_yet_a_success() {
                    let output = CommandOutput::default();

                    assert!(!output.success);
                    assert_eq!(output.status, Some(0));
                    assert!(output.stdout.is_empty() && output.stderr.is_empty());
                    assert_eq!(output.display_lines, 10);
                }

                #[test]
                fn default_replayed_should_succeed_but_returned_by_interceptor_should_fail() {
                    // GIVEN interceptors replaying or returning the default output
                    struct Returning;

                    impl Interceptor<ProcessCommand> for Returning {
                        fn handle<'a>(
                            &'a self,
                            _next: Next<'a, ProcessCommand>,
                        ) -> BoxFuture<'a, Result<CommandOutput>> {
                            Box::pin(std::future::ready(Ok(CommandOutput::default())))
                        }
                    }

                    let replayed = process(std::env::temp_dir());
                    replayed.add_interceptor(crate::interceptor::DryRun);
                    let returned = process(std::env::temp_dir());
                    returned.add_interceptor(Returning);

                    // WHEN running a command through them
                    let replayed = replayed.run_blocking(&shell("exit 1"));
                    let error = returned.run_blocking(&shell("exit 0")).unwrap_err();

                    // THEN only the replayed one gets its success set by the run
                    assert!(replayed.unwrap().success);
                    assert!(
                        error
                            .description()
                            .ends_with("' was not accepted with exit code 0")
                    );
                }
            }
        }

        mod succeeded {
            use super::*;

            #[test]
            fn succeeded_should_return_success_with_stdout() {
                let output = CommandOutput::succeeded("out");

                assert!(output.success);
                assert_eq!(
                    output,
                    CommandOutput {
                        stdout: b"out".to_vec(),
                        ..Default::default()
                    }
                );
            }
        }

        mod failed {
            use super::*;

            #[test]
            fn failed_should_return_failure_with_status_and_stderr() {
                let output = CommandOutput::failed(2, "err");

                assert!(!output.success);
                assert_eq!(
                    output,
                    CommandOutput {
                        status: Some(2),
                        stderr: b"err".to_vec(),
                        ..Default::default()
                    }
                );
            }
        }

        mod stdout_str {
            use super::*;

            #[test]
            fn stdout_str_should_return_utf8_output() {
                let output = CommandOutput {
                    stdout: b"out".to_vec(),
                    ..Default::default()
                };

                assert_eq!(output.stdout_str().unwrap(), "out");
            }

            #[test]
            fn stdout_str_invalid_utf8_should_return_error() {
                let output = CommandOutput {
                    stdout: b"\xFF".to_vec(),
                    ..Default::default()
                };

                let error = output.stdout_str().unwrap_err();

                assert_eq!(error.kind(), crate::ErrorKind::Process);
                assert!(
                    error
                        .description()
                        .starts_with("stdout is not valid UTF-8: ")
                );
                assert!(error.source_as::<std::str::Utf8Error>().is_some());
            }
        }

        mod stderr_str {
            use super::*;

            #[test]
            fn stderr_str_should_return_utf8_output() {
                let output = CommandOutput {
                    stderr: b"err".to_vec(),
                    ..Default::default()
                };

                assert_eq!(output.stderr_str().unwrap(), "err");
            }

            #[test]
            fn stderr_str_invalid_utf8_should_return_error() {
                let output = CommandOutput {
                    stderr: b"\xFF".to_vec(),
                    ..Default::default()
                };

                let error = output.stderr_str().unwrap_err();

                assert!(
                    error
                        .description()
                        .starts_with("stderr is not valid UTF-8: ")
                );
            }
        }

        mod stdout_lossy {
            use super::*;

            #[test]
            fn stdout_lossy_should_replace_invalid_utf8() {
                let output = CommandOutput {
                    stdout: b"out\xFF".to_vec(),
                    ..Default::default()
                };

                assert_eq!(output.stdout_lossy(), "out\u{FFFD}");
            }
        }

        mod stderr_lossy {
            use super::*;

            #[test]
            fn stderr_lossy_should_replace_invalid_utf8() {
                let output = CommandOutput {
                    stderr: b"err\xFF".to_vec(),
                    ..Default::default()
                };

                assert_eq!(output.stderr_lossy(), "err\u{FFFD}");
            }
        }

        mod partial_eq {
            use super::*;

            #[test]
            fn eq_should_ignore_success_and_display_lines() {
                let output = CommandOutput {
                    stdout: b"out".to_vec(),
                    ..Default::default()
                };

                assert_eq!(
                    output,
                    CommandOutput {
                        success: false,
                        display_lines: 3,
                        ..output.clone()
                    }
                );
            }

            #[test]
            fn eq_should_compare_data() {
                let output = CommandOutput {
                    success: true,
                    status: Some(0),
                    signal: None,
                    stdout: b"out".to_vec(),
                    stderr: b"err".to_vec(),
                    duration: Duration::from_secs(1),
                    timed_out: false,
                    display_lines: 10,
                };

                for (different, field) in [
                    (
                        CommandOutput {
                            status: Some(1),
                            ..output.clone()
                        },
                        "status",
                    ),
                    (
                        CommandOutput {
                            signal: Some(9),
                            ..output.clone()
                        },
                        "signal",
                    ),
                    (
                        CommandOutput {
                            stdout: Vec::new(),
                            ..output.clone()
                        },
                        "stdout",
                    ),
                    (
                        CommandOutput {
                            stderr: Vec::new(),
                            ..output.clone()
                        },
                        "stderr",
                    ),
                    (
                        CommandOutput {
                            duration: Duration::ZERO,
                            ..output.clone()
                        },
                        "duration",
                    ),
                    (
                        CommandOutput {
                            timed_out: true,
                            ..output.clone()
                        },
                        "timed_out",
                    ),
                ] {
                    assert_ne!(output, different, "field: {field}");
                }
            }
        }

        mod std_fmt_display {
            use super::*;

            #[test]
            fn fmt_should_show_status_duration_and_output() {
                let output = CommandOutput {
                    success: false,
                    status: Some(1),
                    signal: None,
                    stdout: b"out".to_vec(),
                    stderr: b"err".to_vec(),
                    duration: Duration::from_millis(5),
                    timed_out: false,
                    display_lines: 10,
                };

                assert_eq!(
                    output.to_string(),
                    "status: 1\nduration: 5ms\nstdout:\nout\nstderr:\nerr"
                );
            }

            #[test]
            fn fmt_long_output_should_show_last_lines() {
                let output = CommandOutput {
                    stdout: (1..=12)
                        .map(|line| format!("{line}\n"))
                        .collect::<String>()
                        .into_bytes(),
                    ..Default::default()
                };

                assert_eq!(
                    output.to_string(),
                    "status: 0\nduration: 0ns\nstdout:\n(2 lines omitted)\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n\nstderr:\n"
                );
            }

            #[test]
            fn fmt_without_status_should_show_none() {
                let output = CommandOutput {
                    status: None,
                    ..Default::default()
                };

                assert_eq!(
                    output.to_string(),
                    "status: none\nduration: 0ns\nstdout:\n\nstderr:\n"
                );
            }

            #[test]
            fn fmt_with_signal_should_show_signal() {
                let output = CommandOutput {
                    status: None,
                    signal: Some(9),
                    ..Default::default()
                };

                assert_eq!(
                    output.to_string(),
                    "status: signal 9\nduration: 0ns\nstdout:\n\nstderr:\n"
                );
            }
        }

        mod outcome {
            use super::*;

            mod success {
                use super::*;

                #[test]
                fn success_should_return_success_field() {
                    for success in [true, false] {
                        let output = CommandOutput {
                            success,
                            ..Default::default()
                        };

                        assert_eq!(Outcome::success(&output), success);
                    }
                }
            }

            mod set_success {
                use super::*;

                #[test]
                fn set_success_should_set_success_field() {
                    let mut output = CommandOutput::default();

                    output.set_success(true);

                    assert!(output.success);
                }
            }
        }
    }
}
