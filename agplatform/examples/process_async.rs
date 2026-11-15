//! The checks of a project (format, lint, test) run concurrently as async
//! child processes on a tokio runtime.
//!
//! The checks take the platform shared in an `Arc`, so that they can be
//! spawned as tasks, and their tests share a `TestPlatform` the same way.
//! `TestProcess` answers async runs at once, so the tests need only a
//! single-threaded runtime and never wait for a timeout, unless an output
//! is set with `TestProcess::set_pending`. Such a run is held until its
//! `TestProcessGate` is released, so a test can check what happens while
//! a check is still running, e.g. that failing fast cancels it.
//!
//! ```bash
//! cargo run --example process_async --features process,tokio
//! cargo run --example process_async --features process,tokio -- --fail-fast
//! ```

use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use agplatform::CommandExt;
use agplatform::Env;
use agplatform::Platform;
use tokio::task::JoinSet;

#[derive(Clone, Copy, Debug)]
struct Check {
    name: &'static str,
    program: &'static str,
    args: &'static [&'static str],
}

#[derive(Debug, PartialEq)]
enum Verdict {
    Passed,
    Failed(String),
    TimedOut,
}

const CHECKS: [Check; 3] = [
    Check {
        name: "format",
        program: "cargo",
        args: &["fmt", "--check"],
    },
    Check {
        name: "lint",
        program: "cargo",
        args: &["clippy", "--", "-D", "warnings"],
    },
    Check {
        name: "test",
        program: "cargo",
        args: &["test"],
    },
];

async fn run_check(platform: &impl Platform, check: Check, timeout: Duration) -> Verdict {
    let mut command = Command::new(check.program);
    command.args(check.args);

    match command.timeout(timeout).run_async(platform.process()).await {
        Ok(_) => Verdict::Passed,
        Err(error) => match error.command_output() {
            Some(output) if output.timed_out => Verdict::TimedOut,
            Some(output) => Verdict::Failed(output.stderr_lossy().into_owned()),
            None => Verdict::Failed(error.to_string()),
        },
    }
}

fn spawn_checks<P: Platform + 'static>(
    platform: &Arc<P>,
    checks: &[Check],
    timeout: Duration,
) -> JoinSet<(usize, Verdict)> {
    let mut tasks = JoinSet::new();

    for (index, &check) in checks.iter().enumerate() {
        let platform = platform.clone();
        tasks.spawn(async move { (index, run_check(platform.as_ref(), check, timeout).await) });
    }

    tasks
}

/// Runs all `checks` concurrently and returns their verdicts in their
/// order.
async fn run_all<P: Platform + 'static>(
    platform: &Arc<P>,
    checks: &[Check],
    timeout: Duration,
) -> Vec<(&'static str, Verdict)> {
    let mut verdicts = spawn_checks(platform, checks, timeout).join_all().await;
    verdicts.sort_by_key(|(index, _)| *index);
    verdicts
        .into_iter()
        .map(|(index, verdict)| (checks[index].name, verdict))
        .collect()
}

/// Runs `checks` concurrently until one of them does not pass and returns
/// it. Dropping the remaining tasks cancels their runs, which kills their
/// processes.
async fn run_until_failure<P: Platform + 'static>(
    platform: &Arc<P>,
    checks: &[Check],
    timeout: Duration,
) -> Option<(&'static str, Verdict)> {
    let mut tasks = spawn_checks(platform, checks, timeout);

    while let Some(joined) = tasks.join_next().await {
        match joined.expect("checks should not panic") {
            (_, Verdict::Passed) => {}
            (index, verdict) => return Some((checks[index].name, verdict)),
        }
    }

    None
}

fn main() {
    let platform = Arc::new(agplatform::platform());
    let fail_fast = platform.env().args().any(|arg| arg == "--fail-fast");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime should start");
    let timeout = Duration::from_secs(600);

    runtime.block_on(async {
        if fail_fast {
            println!("{:?}", run_until_failure(&platform, &CHECKS, timeout).await);
        } else {
            for (name, verdict) in run_all(&platform, &CHECKS, timeout).await {
                println!("{name}: {verdict:?}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::CommandOutput;
    use agplatform::Process;
    use agplatform::ProcessCommand;
    use agplatform::interceptor::Interceptor;
    use agplatform::interceptor::Run;
    use agplatform::test_platform::test_platform;
    use std::sync::Mutex;

    fn block_on<T>(future: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(future)
    }

    fn exit(status: i32, stderr: &str) -> CommandOutput {
        CommandOutput {
            status: Some(status),
            stderr: stderr.as_bytes().to_vec(),
            ..Default::default()
        }
    }

    mod run_check {
        use super::*;

        #[test]
        fn run_check_should_report_verdict_of_process() {
            // GIVEN a platform where fmt passes, clippy fails and tests hang
            let platform = test_platform();
            platform.process.set_output("cargo fmt", exit(0, ""));
            platform
                .process
                .set_output("cargo clippy", exit(101, "error: unused variable"));
            platform.process.set_output(
                "cargo test",
                CommandOutput {
                    duration: Duration::from_secs(3600),
                    ..exit(0, "")
                },
            );

            // WHEN running the checks with a minute each
            let verdicts = block_on(async {
                let mut verdicts = Vec::new();
                for check in CHECKS {
                    verdicts.push(run_check(&platform, check, Duration::from_secs(60)).await);
                }
                verdicts
            });

            // THEN each gets the verdict of its process
            assert_eq!(
                verdicts,
                [
                    Verdict::Passed,
                    Verdict::Failed("error: unused variable".to_string()),
                    Verdict::TimedOut
                ]
            );
        }

        #[test]
        fn run_check_missing_program_should_fail() {
            let platform = test_platform();
            platform
                .process
                .set_io_error("cargo", std::io::ErrorKind::NotFound);

            let verdict = block_on(run_check(&platform, CHECKS[0], Duration::from_secs(60)));

            assert!(matches!(verdict, Verdict::Failed(message) if message.contains("'cargo'")));
        }
    }

    mod run_all {
        use super::*;

        #[test]
        fn run_all_should_run_every_check_and_keep_their_order() {
            // GIVEN a platform shared by the tasks where only the tests fail
            let platform = Arc::new(test_platform());
            platform.process.set_output("cargo", exit(0, ""));
            platform
                .process
                .set_output("cargo test", exit(101, "test failed"));

            // WHEN running all checks concurrently
            let verdicts = block_on(run_all(&platform, &CHECKS, Duration::from_secs(60)));

            // THEN every check ran, in any order, and the verdicts keep the
            // order of the checks
            let mut calls = platform.process.calls();
            calls.sort();
            assert_eq!(
                calls,
                [
                    "cargo clippy -- -D warnings",
                    "cargo fmt --check",
                    "cargo test"
                ]
            );
            assert_eq!(
                verdicts,
                [
                    ("format", Verdict::Passed),
                    ("lint", Verdict::Passed),
                    ("test", Verdict::Failed("test failed".to_string()))
                ]
            );
        }
    }

    mod run_until_failure {
        use super::*;

        #[test]
        fn run_until_failure_should_cancel_checks_still_running() {
            // GIVEN the lint failing and the tests running until cancelled,
            // with an interceptor of the cancelled runs
            #[derive(Clone, Default)]
            struct Cancelled(Arc<Mutex<Vec<String>>>);

            impl Interceptor<ProcessCommand> for Cancelled {
                fn cancelled(&self, run: &Run<ProcessCommand>) {
                    self.0.lock().unwrap().push(run.subject().command_line());
                }
            }

            let cancelled = Cancelled::default();
            let platform = test_platform();
            platform.process().add_interceptor(cancelled.clone());
            platform.process.set_output("cargo fmt", exit(0, ""));
            let lint = platform
                .process
                .set_pending("cargo clippy", exit(101, "warning"));
            let _tests = platform.process.set_pending("cargo test", exit(0, ""));
            let platform = Arc::new(platform);

            let failure = block_on(async {
                // WHEN the lint fails while all checks are running
                let checking = tokio::spawn({
                    let platform = platform.clone();
                    async move { run_until_failure(&platform, &CHECKS, Duration::from_secs(60)).await }
                });
                tokio::time::timeout(Duration::from_secs(10), async {
                    while platform.process.calls().len() < 3 {
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .expect("all checks should start");
                lint.release();
                checking.await.unwrap()
            });

            // THEN its failure is returned and the tests still running are
            // cancelled, which kills their process
            assert_eq!(
                failure,
                Some(("lint", Verdict::Failed("warning".to_string())))
            );
            assert_eq!(*cancelled.0.lock().unwrap(), ["cargo test"]);
        }

        #[test]
        fn run_until_failure_should_return_failed_check() {
            let platform = Arc::new(test_platform());
            platform.process.set_output("cargo", exit(0, ""));
            platform
                .process
                .set_output("cargo clippy", exit(101, "warning"));

            let failure = block_on(run_until_failure(
                &platform,
                &CHECKS,
                Duration::from_secs(60),
            ));

            assert_eq!(
                failure,
                Some(("lint", Verdict::Failed("warning".to_string())))
            );
        }

        #[test]
        fn run_until_failure_all_passing_should_return_none() {
            let platform = Arc::new(test_platform());
            platform.process.set_output("cargo", exit(0, ""));

            let failure = block_on(run_until_failure(
                &platform,
                &CHECKS,
                Duration::from_secs(60),
            ));

            assert_eq!(failure, None);
            assert_eq!(platform.process.calls().len(), 3);
        }
    }
}
