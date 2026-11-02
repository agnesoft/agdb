//! A release helper running `git` and `cargo` as child processes.
//!
//! The helper takes `&impl Process`, so its tests run against a
//! `TestProcess` that never spawns anything. Each test sets the output of
//! the commands it expects and then checks which commands ran, with which
//! arguments, input and options. That covers retries, timeouts and missing
//! programs, all deterministic and without waiting.
//!
//! ```bash
//! cargo run --example process --features process -- branch
//! cargo run --example process --features process -- build
//! ```

use std::io::Write;
use std::process::Command;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use agplatform::CommandExt;
use agplatform::CommandOutput;
use agplatform::Env;
use agplatform::OutputStream;
use agplatform::Platform;
use agplatform::Process;
use agplatform::ProcessCommand;
use agplatform::interceptor::BoxFuture;
use agplatform::interceptor::Chunk;
use agplatform::interceptor::Interceptor;
use agplatform::interceptor::Next;
use agplatform::interceptor::Retry;
use agplatform::interceptor::Run;

/// Calls the process directly, which is equivalent to running the command
/// with `CommandExt::run_blocking` like the other functions.
fn current_branch(process: &impl Process) -> agplatform::Result<String> {
    let output =
        process.run_blocking(Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]))?;
    Ok(output.stdout_str()?.trim().to_string())
}

/// Commits the staged changes, passing `message` on the standard input so
/// that it may hold any characters.
fn commit(process: &impl Process, message: &str) -> agplatform::Result<()> {
    Command::new("git")
        .args(["commit", "--file", "-"])
        .input(message)
        .run_blocking(process)?;
    Ok(())
}

/// Pushes the current branch, retrying only the failures caused by the
/// network, not e.g. a rejected push, around the interceptors of `process`.
/// Each attempt is limited to a minute.
fn push(process: &impl Process) -> agplatform::Result<()> {
    let mut interceptors = process.interceptors();
    interceptors.insert(
        0,
        Retry::<ProcessCommand>::new(3, Duration::from_secs(5)).when(|_, result| {
            result
                .as_ref()
                .is_ok_and(|output| output.stderr_lossy().contains("Could not resolve host"))
        }),
    );

    Command::new("git")
        .arg("push")
        .timeout(Duration::from_secs(60))
        .interceptors(interceptors)
        .run_blocking(process)?;
    Ok(())
}

/// Returns whether `program` can be started, i.e. is installed.
fn installed(process: &impl Process, program: &str) -> bool {
    match Command::new(program)
        .arg("--version")
        .silent()
        .run_blocking(process)
    {
        Ok(_) => true,
        Err(error) => error.command_output().is_some(),
    }
}

/// Builds the project, showing its output live through the interceptors of
/// `process`.
fn build(process: &impl Process) -> agplatform::Result<CommandOutput> {
    Command::new("cargo")
        .args(["build", "--release"])
        .stream()
        .run_blocking(process)
}

/// Writes the streamed output of the commands to `W` with each line
/// prefixed by the program, instead of the default
/// [`StdStream`](agplatform::StdStream). The chunks are as read, so the
/// start of a line is kept until its end arrives, the next attempt of the
/// run starts or the run ends.
struct Prefixed<W>(Arc<Mutex<Lines<W>>>);

/// The output and the unfinished lines of the runs by their id, attempt
/// and stream.
struct Lines<W> {
    out: W,
    unfinished: Vec<(u64, u32, OutputStream, Vec<u8>)>,
}

impl<W: Write> Prefixed<W> {
    fn new(out: W) -> Self {
        Self(Arc::new(Mutex::new(Lines {
            out,
            unfinished: Vec::new(),
        })))
    }
}

impl<W: Write> Lines<W> {
    /// Writes the unfinished lines of `run` from the attempts before
    /// `attempt`.
    fn finish(&mut self, run: &Run<ProcessCommand>, attempt: u32) {
        let finished = self
            .unfinished
            .extract_if(.., |(id, other, _, _)| *id == run.id() && *other < attempt);

        for (_, _, _, line) in finished.filter(|(_, _, _, line)| !line.is_empty()) {
            write_line(&mut self.out, run, &line);
        }
    }
}

impl<W> Clone for Prefixed<W> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<W: Write + Send + 'static> Interceptor<ProcessCommand> for Prefixed<W> {
    fn handle<'a>(
        &'a self,
        mut next: Next<'a, ProcessCommand>,
    ) -> BoxFuture<'a, agplatform::Result<CommandOutput>> {
        Box::pin(async move {
            let result = next.attempt().await;
            self.0.lock().unwrap().finish(next.run(), u32::MAX);
            result
        })
    }

    fn chunk(&self, chunk: Chunk<ProcessCommand>) -> agplatform::Result {
        if chunk.run().subject().options().stream {
            let (run, stream) = (chunk.run(), chunk.stream());
            let mut lines = self.0.lock().unwrap();
            lines.finish(run, run.attempt());
            let Lines { out, unfinished } = &mut *lines;
            let index = unfinished
                .iter()
                .position(|(id, _, other, _)| *id == run.id() && *other == stream)
                .unwrap_or_else(|| {
                    unfinished.push((run.id(), run.attempt(), stream, Vec::new()));
                    unfinished.len() - 1
                });
            let line = &mut unfinished[index].3;
            line.extend_from_slice(chunk.data());

            while let Some(end) = line.iter().position(|&byte| byte == b'\n') {
                let rest = line.split_off(end + 1);
                write_line(out, run, &line[..end]);
                *line = rest;
            }
        }

        chunk.forward()
    }
}

/// Writes `line` of `run` prefixed by its program.
fn write_line(out: &mut impl Write, run: &Run<ProcessCommand>, line: &[u8]) {
    let line = String::from_utf8_lossy(line);
    let _ = writeln!(
        out,
        "[{}] {}",
        run.subject().program(),
        line.trim_end_matches('\r')
    );
}

fn main() -> agplatform::Result<()> {
    let platform = agplatform::platform();
    let process = platform.process();
    process.clear_interceptors();
    process.add_interceptor(Prefixed::new(std::io::stdout()));
    let args: Vec<String> = platform
        .env()
        .args()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["branch"] => println!("{}", current_branch(process)?),
        ["commit", message] => commit(process, message)?,
        ["push"] => push(process)?,
        ["build"] => println!("built in {:?}", build(process)?.duration),
        _ => println!(
            "usage: process branch|commit <message>|push|build (git installed: {})",
            installed(process, "git")
        ),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::TestProcess;

    fn exit(status: i32, stderr: &str) -> CommandOutput {
        CommandOutput {
            status: Some(status),
            stderr: stderr.as_bytes().to_vec(),
            ..Default::default()
        }
    }

    mod current_branch {
        use super::*;

        #[test]
        fn current_branch_should_return_trimmed_stdout_of_git() {
            let process = TestProcess::new();
            process.set_output(
                "git rev-parse",
                CommandOutput {
                    status: Some(0),
                    stdout: b"main\n".to_vec(),
                    ..Default::default()
                },
            );

            let branch = current_branch(&process).unwrap();

            assert_eq!(branch, "main");
            assert_eq!(process.calls(), ["git rev-parse --abbrev-ref HEAD"]);
        }

        #[test]
        fn current_branch_outside_repository_should_fail_with_git_output() {
            let process = TestProcess::new();
            process.set_output("git", exit(128, "fatal: not a git repository"));

            let error = current_branch(&process).unwrap_err();

            let output = error.command_output().unwrap();
            assert_eq!(output.status, Some(128));
            assert_eq!(output.stderr_str().unwrap(), "fatal: not a git repository");
        }
    }

    mod commit {
        use super::*;

        #[test]
        fn commit_should_pass_message_on_stdin() {
            let process = TestProcess::new();
            process.set_output("git commit", exit(0, ""));

            commit(&process, "Fix \"quotes\" and $vars").unwrap();

            let call = &process.call_details()[0];
            assert_eq!(call.command_line, "git commit --file -");
            assert_eq!(
                call.options.input.as_deref(),
                Some(b"Fix \"quotes\" and $vars".as_slice())
            );
        }
    }

    mod push {
        use super::*;

        #[test]
        fn push_should_retry_network_failures() {
            // GIVEN a push failing twice to reach the remote
            let process = TestProcess::new();
            let offline = exit(128, "fatal: Could not resolve host: github.com");
            process.set_outputs("git push", [offline.clone(), offline, exit(0, "")]);

            // WHEN pushing
            let result = push(&process);

            // THEN it succeeds on the third attempt, without waiting for the
            // retry delays
            assert!(result.is_ok());
            assert_eq!(process.calls(), ["git push", "git push", "git push"]);
        }

        #[test]
        fn push_rejected_should_fail_without_retrying() {
            let process = TestProcess::new();
            process.set_output(
                "git push",
                exit(1, "! [rejected] main -> main (fetch first)"),
            );

            let error = push(&process).unwrap_err();

            assert!(
                error
                    .command_output()
                    .unwrap()
                    .stderr_lossy()
                    .contains("rejected")
            );
            assert_eq!(process.calls().len(), 1);
        }

        #[test]
        fn push_hanging_should_time_out() {
            // GIVEN a push that would take longer than its timeout
            let process = TestProcess::new();
            process.set_output(
                "git push",
                CommandOutput {
                    status: Some(0),
                    duration: Duration::from_secs(3600),
                    ..Default::default()
                },
            );

            // WHEN pushing
            let error = push(&process).unwrap_err();

            // THEN it times out at once without being retried
            assert!(error.command_output().unwrap().timed_out);
            assert_eq!(
                process.call_details()[0].options.timeout,
                Some(Duration::from_secs(60))
            );
            assert_eq!(process.calls().len(), 1);
        }
    }

    mod installed {
        use super::*;

        #[test]
        fn installed_should_tell_whether_program_starts() {
            let process = TestProcess::new();
            process.set_output("git", exit(0, ""));
            process.set_output("hg", exit(1, ""));
            process.set_io_error("svn", std::io::ErrorKind::NotFound);

            for (program, expected) in [("git", true), ("hg", true), ("svn", false)] {
                assert_eq!(installed(&process, program), expected, "program: {program}");
            }
        }
    }

    mod build {
        use super::*;

        #[test]
        fn build_should_stream_output() {
            let process = TestProcess::new();
            process.set_output(
                "cargo build",
                CommandOutput {
                    status: Some(0),
                    stderr: b"Compiling app\nFinished".to_vec(),
                    ..Default::default()
                },
            );

            build(&process).unwrap();

            assert!(process.call_details()[0].options.stream);
        }
    }

    mod prefixed {
        use super::*;

        mod interceptor {
            use super::*;

            mod handle {
                #[test]
                fn handle_invalid_run_should_fail_in_one_attempt() {
                    // WHY: an example cannot call the contract cases, which are test
                    // code of agplatform.
                }

                #[test]
                fn handle_replayed_run_should_return_replay_output_once() {
                    // WHY: an example cannot call the contract cases, which are test
                    // code of agplatform.
                }
            }

            mod chunk {
                use super::*;

                #[test]
                fn chunk_should_pass_chunk_on_unchanged() {
                    // WHY: an example cannot call the contract cases, which are test
                    // code of agplatform.
                }

                #[test]
                fn chunk_should_prefix_lines_of_streamed_runs_with_program() {
                    // GIVEN the interceptor installed on the process
                    let prefixed = Prefixed::new(Vec::new());
                    let process = TestProcess::new();
                    process.clear_interceptors();
                    process.add_interceptor(prefixed.clone());
                    process.set_output(
                        "cargo",
                        CommandOutput {
                            status: Some(0),
                            stdout: b"Compiling app\r\nFinish".to_vec(),
                            stderr: b"warning: unused".to_vec(),
                            ..Default::default()
                        },
                    );

                    // WHEN a command runs with and without streaming its output
                    Command::new("cargo")
                        .stream()
                        .run_blocking(&process)
                        .unwrap();
                    Command::new("cargo").run_blocking(&process).unwrap();

                    // THEN each line of the streamed one is written with the
                    // program, the unfinished ones when it ends
                    assert_eq!(
                        String::from_utf8(prefixed.0.lock().unwrap().out.clone()).unwrap(),
                        "[cargo] Compiling app\n[cargo] Finish\n[cargo] warning: unused\n"
                    );
                }

                #[test]
                fn chunk_retried_should_end_unfinished_line_of_failed_attempt() {
                    // GIVEN a retry inside the interceptor
                    let prefixed = Prefixed::new(Vec::new());
                    let mut interceptors = agplatform::interceptor::Interceptors::new();
                    interceptors.push(prefixed.clone());
                    interceptors.push(Retry::new(1, Duration::ZERO));
                    let process = TestProcess::new();
                    let output = |status, stdout: &[u8]| CommandOutput {
                        status: Some(status),
                        stdout: stdout.to_vec(),
                        ..Default::default()
                    };
                    process.set_outputs("cargo", [output(1, b"Fail"), output(0, b"ok\n")]);

                    // WHEN the first attempt fails in the middle of a line
                    Command::new("cargo")
                        .stream()
                        .interceptors(interceptors)
                        .run_blocking(&process)
                        .unwrap();

                    // THEN its line ends before the output of the retry
                    assert_eq!(
                        String::from_utf8(prefixed.0.lock().unwrap().out.clone()).unwrap(),
                        "[cargo] Fail\n[cargo] ok\n"
                    );
                }
            }
        }
    }
}
