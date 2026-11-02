use std::sync::Arc;

use super::{BoxFuture, Interceptor, Next, Outcome, Run, Subject};
use crate::Result;

/// The [`Interceptor`] accepting the failed outputs of the runs for which
/// its condition returns `true`, making them a success (see
/// [`Outcome::set_success`]), e.g. a `404` of a request expected to find
/// nothing. A condition depending on the run reads a value set on it, see
/// [`Run::get`].
///
/// An output it accepts is a success for the interceptors before it in the
/// chain, e.g. not retried by a [`Retry`](super::Retry) before it.
///
/// # Examples
///
/// ```rust
/// use agplatform::interceptor::Accept;
/// use agplatform::{HttpRequest, Platform, Request};
///
/// /// Set on a request expected to find nothing.
/// struct MayBeMissing;
///
/// let platform = agplatform::platform();
/// platform.request().add_interceptor(Accept::<HttpRequest>::when(|run, output| {
///     output.status == 404 && run.get::<MayBeMissing>().is_some()
/// }));
/// ```
pub struct Accept<S: Subject> {
    condition: Arc<AcceptCondition<S>>,
}

type AcceptCondition<S> = dyn Fn(&Run<S>, &<S as Subject>::Output) -> bool + Send + Sync;

impl<S: Subject> Accept<S> {
    /// Creates the interceptor accepting the failed outputs for which
    /// `condition` returns `true`.
    pub fn when<F>(condition: F) -> Self
    where
        F: Fn(&Run<S>, &S::Output) -> bool + Send + Sync + 'static,
    {
        Self {
            condition: Arc::new(condition),
        }
    }
}

impl<S: Subject> Clone for Accept<S> {
    fn clone(&self) -> Self {
        Self {
            condition: self.condition.clone(),
        }
    }
}

impl<S: Subject> std::fmt::Debug for Accept<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Accept")
    }
}

impl<S: Subject> Interceptor<S> for Accept<S> {
    fn handle<'a>(&'a self, mut next: Next<'a, S>) -> BoxFuture<'a, Result<S::Output>> {
        Box::pin(async move {
            let mut output = next.attempt().await?;

            if !output.success() && (self.condition)(next.run(), &output) {
                output.set_success(true);
            }

            Ok(output)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interceptor::Retry;
    use crate::interceptor::tests::*;

    mod accept {
        use super::*;

        mod when {
            use super::*;

            #[test]
            fn when_should_accept_failures_meeting_condition() {
                for (status, success) in [(1, true), (2, false)] {
                    let script = Script::new([Ok(done(status))]);
                    let accept = Accept::when(|_, output: &Done| output.status == 1);

                    let output = run(&script, &chain(vec![Arc::new(accept)]), task("x")).unwrap();

                    assert_eq!(output.success, success, "status: {status}");
                }
            }

            #[test]
            fn when_should_read_values_of_run() {
                struct Expected(i32);
                let accept = Accept::when(|run: &Run<Task>, output: &Done| {
                    run.get::<Expected>()
                        .is_some_and(|expected| expected.0 == output.status)
                });

                let output = run(
                    &Script::new([Ok(done(4))]),
                    &chain(vec![Arc::new(accept)]),
                    task("x").with(Expected(4)),
                )
                .unwrap();

                assert!(output.success);
            }
        }

        mod clone {
            use super::*;

            #[test]
            fn clone_should_share_condition() {
                let accept = Accept::<Task>::when(|_, _| true);

                let clone = accept.clone();

                assert!(Arc::ptr_eq(&accept.condition, &clone.condition));
            }
        }

        mod std_fmt_debug {
            use super::*;

            #[test]
            fn fmt_should_show_name() {
                assert_eq!(format!("{:?}", Accept::<Task>::when(|_, _| true)), "Accept");
            }
        }

        mod interceptor {
            use super::*;

            mod handle {
                use super::*;

                #[test]
                fn handle_replayed_run_should_return_replay_output_once() {
                    crate::interceptor::tests::interceptor::handle::handle_replayed_run_should_return_replay_output_once(
                        Arc::new(Accept::when(|_, output: &Done| output.status == 2)),
                        &Script::default(),
                        task("x").with_replay_output(data(1, "set")),
                    );
                }

                #[test]
                fn handle_invalid_run_should_fail_in_one_attempt() {
                    let mut task = task("x");
                    task.subject_mut().invalid = true;

                    crate::interceptor::tests::interceptor::handle::handle_invalid_run_should_fail_in_one_attempt(
                        Arc::new(Accept::when(|_, _: &Done| true)),
                        &Script::default(),
                        task,
                    );
                }

                #[test]
                fn handle_accepted_failure_should_not_be_retried() {
                    let script = Script::new([Ok(done(1)), Ok(done(0))]);
                    let interceptors = chain(vec![
                        Arc::new(Retry::new(1, ms(1))),
                        Arc::new(Accept::when(|_, _: &Done| true)),
                    ]);

                    let output = run(&script, &interceptors, task("x")).unwrap();

                    assert_eq!(output.status, 1);
                    assert_eq!(script.calls().len(), 1);
                }

                #[test]
                fn handle_error_should_be_returned() {
                    let script = Script::new([Err(crate::Error::process("io".to_string()))]);
                    let accept = Accept::when(|_, _: &Done| true);

                    let error =
                        run(&script, &chain(vec![Arc::new(accept)]), task("x")).unwrap_err();

                    assert_eq!(error.description(), "io");
                }
            }

            mod chunk {
                use super::*;

                #[test]
                fn chunk_should_pass_chunk_on_unchanged() {
                    crate::interceptor::tests::interceptor::chunk::chunk_should_pass_chunk_on_unchanged(
                        Arc::new(Accept::when(|_, _: &Done| true)),
                        &task("x"),
                        1,
                    );
                }
            }
        }
    }
}
