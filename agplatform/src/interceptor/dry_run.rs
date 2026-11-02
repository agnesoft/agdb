use super::{BoxFuture, Interceptor, Next, Subject};
use crate::Result;

/// The output [`DryRun`] sets on a run instead of running it, set on the
/// run with [`Run::with`](super::Run::with).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DryRunOutput<T>(pub T);

/// The [`Interceptor`] of a dry run, setting the replay output of the runs
/// (see [`Next::set_replay_output`]) instead of running them: the
/// [`DryRunOutput`] set on the run (see [`Run::with`](super::Run::with)), or
/// the default output, unless the run is replayed already.
#[derive(Clone, Copy, Debug, Default)]
pub struct DryRun;

impl<S: Subject> Interceptor<S> for DryRun
where
    S::Output: Default,
{
    fn handle<'a>(&'a self, mut next: Next<'a, S>) -> BoxFuture<'a, Result<S::Output>> {
        if !next.run().is_replay() {
            let output = next
                .run()
                .get::<DryRunOutput<S::Output>>()
                .map_or_else(S::Output::default, |output| output.0.clone());
            next.set_replay_output(output);
        }

        next.forward()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interceptor::tests::*;
    use std::sync::Arc;

    mod dry_run {
        use super::*;

        mod interceptor {
            use super::*;

            mod handle {
                use super::*;

                #[test]
                fn handle_should_set_default_output() {
                    let script = Script::default();

                    let output = run(&script, &chain(vec![Arc::new(DryRun)]), task("x")).unwrap();

                    assert_eq!(
                        output,
                        Done {
                            success: true,
                            ..Default::default()
                        }
                    );
                    assert!(script.calls().is_empty());
                }

                #[test]
                fn handle_should_set_dry_run_output_of_run() {
                    let task = task("x").with(DryRunOutput(data(0, "given")));

                    let output =
                        run(&Script::default(), &chain(vec![Arc::new(DryRun)]), task).unwrap();

                    assert_eq!(output.data, b"given");
                }

                #[test]
                fn handle_should_pass_output_to_sink_as_if_it_ran() {
                    let task = task("x").with(DryRunOutput(data(0, "given")));
                    let mut sunk = Vec::new();

                    run_to(
                        &Script::default(),
                        &chain(vec![Arc::new(DryRun)]),
                        task,
                        &mut sunk,
                    )
                    .unwrap();

                    assert_eq!(sunk, [(1, b"given".to_vec())]);
                }

                #[test]
                fn handle_failing_output_under_retry_should_not_be_retried() {
                    let task = task("x").with(DryRunOutput(done(1)));
                    let interceptors = chain(vec![
                        Arc::new(crate::interceptor::Retry::new(2, ms(1))),
                        Arc::new(DryRun),
                    ]);
                    let script = Script::default();

                    let output = run(&script, &interceptors, task).unwrap();

                    assert!(!output.success);
                    assert!(script.sleeps().is_empty());
                }

                #[test]
                fn handle_replayed_run_should_return_replay_output_once() {
                    crate::interceptor::tests::interceptor::handle::handle_replayed_run_should_return_replay_output_once(
                        Arc::new(DryRun),
                        &Script::default(),
                        task("x").with_replay_output(data(1, "set")),
                    );
                }

                #[test]
                fn handle_invalid_run_should_fail_in_one_attempt() {
                    let mut task = task("x");
                    task.subject_mut().invalid = true;

                    crate::interceptor::tests::interceptor::handle::handle_invalid_run_should_fail_in_one_attempt(
                        Arc::new(DryRun),
                        &Script::default(),
                        task,
                    );
                }
            }

            mod chunk {
                use super::*;

                #[test]
                fn chunk_should_pass_chunk_on_unchanged() {
                    crate::interceptor::tests::interceptor::chunk::chunk_should_pass_chunk_on_unchanged(
                        Arc::new(DryRun),
                        &task("x"),
                        1,
                    );
                }
            }
        }
    }
}
