use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use super::{BoxFuture, Interceptor, Next, Outcome, Run, Subject};
use crate::Result;

/// The [`Interceptor`] retrying the failed attempts of the runs, up to
/// `retries` times after `delay` (see [`Next::sleep`]) while they fail,
/// i.e. their output is not a success or they return an error, unless
/// [`when`](Self::when) decides otherwise. Not after a part of the output
/// was passed to the sink, see [`Run::streamed`], nor when the run is
/// replayed, see [`Run::is_replay`], which would replay the same output,
/// nor when the attempt was rejected, see [`Run::rejected`], which would
/// fail the same way.
///
/// The delay is multiplied by the [`backoff`](Self::backoff) after each
/// retry and limited by the [`max_delay`](Self::max_delay). An output that
/// asks to wait (see [`retry_after`](Self::retry_after)) is retried no
/// sooner than it asks, or not at all if it asks to wait longer than the
/// `max_delay`.
///
/// With a [`deadline`](Self::deadline) the retries also stop once waiting
/// before the next one would end past it.
///
/// It runs again all interceptors after it in the chain, so it is usually
/// the outermost one. Its decision is public, see
/// [`next_delay`](Self::next_delay), e.g. for an interceptor building a
/// `Retry` for each run from a policy set on it.
pub struct Retry<S: Subject> {
    retries: u32,
    delay: Duration,
    backoff: u32,
    max_delay: Duration,
    deadline: Option<Duration>,
    condition: Option<Arc<RetryCondition<S>>>,
    after: Option<Arc<RetryAfter<S>>>,
}

/// The retries of a run so far, kept from its first attempt for
/// [`Retry::next_delay`].
#[derive(Clone, Debug)]
pub struct RetryState {
    retried: u32,
    started: Instant,
    waited: Duration,
}

type RetryCondition<S> = dyn Fn(&Run<S>, &Result<<S as Subject>::Output>) -> bool + Send + Sync;

type RetryAfter<S> = dyn Fn(&<S as Subject>::Output) -> Option<Duration> + Send + Sync;

impl<S: Subject> Retry<S> {
    /// Creates the interceptor retrying the failed attempts up to `retries`
    /// times after `delay`.
    pub fn new(retries: u32, delay: Duration) -> Self {
        Self {
            retries,
            delay,
            backoff: 1,
            max_delay: MAX_DELAY,
            deadline: None,
            condition: None,
            after: None,
        }
    }

    /// Retries only while the time since the first attempt, and the wait
    /// before the next one, end within `deadline`: no wait, also one asked
    /// for (see [`retry_after`](Self::retry_after)), is started that would
    /// end past it. E.g. a polling loop retrying until a resource is ready,
    /// with `retries` set to [`u32::MAX`].
    ///
    /// The time is the longer of the wall clock elapsed and the waits
    /// asked for, so that a test double not waiting (see [`Next::sleep`])
    /// reaches the deadline as well.
    pub fn deadline(mut self, deadline: Duration) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Multiplies the delay by `factor` after each retry. `1` (the default)
    /// keeps the delay fixed, `2` doubles it.
    pub fn backoff(mut self, factor: u32) -> Self {
        self.backoff = factor;
        self
    }

    /// Waits at most `max` before a retry, limiting the backoff, and gives
    /// up when the output asks to wait longer. 60 s by default.
    pub fn max_delay(mut self, max: Duration) -> Self {
        self.max_delay = max;
        self
    }

    /// Waits before retrying a failed output at least as long as `after`
    /// returns for it, e.g. from a `Retry-After` header.
    pub fn retry_after<F>(mut self, after: F) -> Self
    where
        F: Fn(&S::Output) -> Option<Duration> + Send + Sync + 'static,
    {
        self.after = Some(Arc::new(after));
        self
    }

    /// Retries only the failed attempts for which `condition` returns
    /// `true` instead of all of them.
    pub fn when<F>(mut self, condition: F) -> Self
    where
        F: Fn(&Run<S>, &Result<S::Output>) -> bool + Send + Sync + 'static,
    {
        self.condition = Some(Arc::new(condition));
        self
    }

    /// Returns the delay before retrying the attempt of `run` with `result`
    /// and records the retry in `state`, or `None` if it is not retried,
    /// see [`Retry`]. The caller waits for the delay, e.g. with
    /// [`Next::sleep`], before the next attempt.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::interceptor::{Retry, RetryState, Run};
    /// use agplatform::{CommandOutput, ProcessCommand};
    /// use std::time::Duration;
    ///
    /// let retry = Retry::<ProcessCommand>::new(1, Duration::from_secs(1));
    /// let run = Run::new(ProcessCommand::new(&std::process::Command::new("git")));
    /// let mut state = RetryState::new();
    /// let failed = Ok(CommandOutput::default());
    ///
    /// assert_eq!(retry.next_delay(&run, &failed, &mut state), Some(Duration::from_secs(1)));
    /// assert_eq!(retry.next_delay(&run, &failed, &mut state), None);
    /// assert_eq!(state.retried(), 1);
    /// ```
    pub fn next_delay(
        &self,
        run: &Run<S>,
        result: &Result<S::Output>,
        state: &mut RetryState,
    ) -> Option<Duration> {
        let retried_ok = state.retried < self.retries
            && !result.as_ref().is_ok_and(Outcome::success)
            && !(result.is_err() && run.rejected())
            && !run.streamed()
            && !run.is_replay()
            && self
                .condition
                .as_ref()
                .is_none_or(|condition| condition(run, result));
        let delay = backoff(self.delay, self.backoff, state.retried, self.max_delay);
        let after = self
            .after
            .as_ref()
            .zip(result.as_ref().ok())
            .and_then(|(after, output)| after(output));
        let delay = match after {
            _ if !retried_ok => None,
            Some(after) if after > self.max_delay => None,
            Some(after) => Some(delay.max(after)),
            None => Some(delay),
        }
        .filter(|&delay| {
            self.deadline
                .is_none_or(|deadline| state.elapsed().saturating_add(delay) <= deadline)
        })?;

        state.retried += 1;
        state.waited = state.waited.saturating_add(delay);
        Some(delay)
    }
}

impl RetryState {
    /// Creates the state of a run before its first attempt.
    pub fn new() -> Self {
        Self {
            retried: 0,
            started: Instant::now(),
            waited: Duration::ZERO,
        }
    }

    /// Returns the number of the retries so far.
    pub fn retried(&self) -> u32 {
        self.retried
    }

    /// Returns the time since the first attempt for a deadline: the longer
    /// of the wall clock elapsed and the waits asked for.
    fn elapsed(&self) -> Duration {
        self.started.elapsed().max(self.waited)
    }
}

impl Default for RetryState {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Subject> Clone for Retry<S> {
    fn clone(&self) -> Self {
        Self {
            retries: self.retries,
            delay: self.delay,
            backoff: self.backoff,
            max_delay: self.max_delay,
            deadline: self.deadline,
            condition: self.condition.clone(),
            after: self.after.clone(),
        }
    }
}

impl<S: Subject> std::fmt::Debug for Retry<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Retry")
            .field("retries", &self.retries)
            .field("delay", &self.delay)
            .field("backoff", &self.backoff)
            .field("max_delay", &self.max_delay)
            .field("deadline", &self.deadline)
            .field("condition", &self.condition.is_some())
            .field("retry_after", &self.after.is_some())
            .finish()
    }
}

impl<S: Subject> Interceptor<S> for Retry<S> {
    fn handle<'a>(&'a self, mut next: Next<'a, S>) -> BoxFuture<'a, Result<S::Output>> {
        Box::pin(async move {
            let mut state = RetryState::new();

            loop {
                let result = next.attempt().await;

                match self.next_delay(next.run(), &result, &mut state) {
                    Some(delay) => next.sleep(delay).await,
                    None => return result,
                }
            }
        })
    }
}

/// [internal]
///
/// The longest delay of a retry interceptor by default, see
/// [`Retry::max_delay`].
pub(crate) const MAX_DELAY: Duration = Duration::from_secs(60);

/// [internal]
///
/// Returns the delay before a retry after `retried` retries: `delay`
/// multiplied by `backoff` for each of them, at most `max`.
pub(crate) fn backoff(delay: Duration, backoff: u32, retried: u32, max: Duration) -> Duration {
    delay
        .saturating_mul(backoff.saturating_pow(retried))
        .min(max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interceptor::Chunk;
    use crate::interceptor::tests::*;

    mod retry {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn new_should_retry_failed_attempts_up_to_retries() {
                let script = Script::new([
                    Ok(done(1)),
                    Err(crate::Error::process("io".to_string())),
                    Ok(done(2)),
                ]);

                let output = run(
                    &script,
                    &chain(vec![Arc::new(Retry::new(2, ms(1)))]),
                    task("x"),
                )
                .unwrap();

                assert_eq!(output, done(2));
                assert_eq!(script.calls().len(), 3);
                assert_eq!(script.sleeps(), [ms(1), ms(1)]);
            }

            #[test]
            fn new_zero_retries_should_not_retry_failure() {
                let script = Script::new([Ok(done(1))]);

                let output = run(
                    &script,
                    &chain(vec![Arc::new(Retry::new(0, ms(1)))]),
                    task("x"),
                )
                .unwrap();

                assert_eq!(output, done(1));
                assert_eq!(script.calls().len(), 1);
            }

            #[test]
            fn new_exhausted_retries_should_return_last_result() {
                let script = Script::new([
                    Err(crate::Error::process("first".to_string())),
                    Err(crate::Error::process("last".to_string())),
                    Ok(done(0)),
                ]);

                let error = run(
                    &script,
                    &chain(vec![Arc::new(Retry::new(1, ms(1)))]),
                    task("x"),
                )
                .unwrap_err();

                assert_eq!(error.description(), "last");
                assert_eq!(script.calls().len(), 2);
            }

            #[test]
            fn new_should_not_retry_success() {
                let script = Script::default();

                run(
                    &script,
                    &chain(vec![Arc::new(Retry::new(2, ms(1)))]),
                    task("x"),
                )
                .unwrap();

                assert_eq!(script.calls().len(), 1);
            }
        }

        mod backoff {
            use super::*;

            #[test]
            fn backoff_should_multiply_delay_after_each_retry() {
                let script = Script::new([Ok(done(1)), Ok(done(1)), Ok(done(1))]);

                run(
                    &script,
                    &chain(vec![Arc::new(Retry::new(3, ms(1)).backoff(3))]),
                    task("x"),
                )
                .unwrap();

                assert_eq!(script.sleeps(), [ms(1), ms(3), ms(9)]);
            }

            #[test]
            fn backoff_overflowing_should_wait_max_delay() {
                let script = Script::new([Ok(done(1)), Ok(done(1)), Ok(done(1))]);
                let retry = Retry::new(3, ms(1)).backoff(u32::MAX).max_delay(ms(5));

                run(&script, &chain(vec![Arc::new(retry)]), task("x")).unwrap();

                assert_eq!(script.sleeps(), [ms(1), ms(5), ms(5)]);
            }
        }

        mod max_delay {
            use super::*;

            #[test]
            fn max_delay_should_limit_backoff() {
                let script = Script::new([Ok(done(1)), Ok(done(1)), Ok(done(1))]);
                let retry = Retry::new(3, ms(2)).backoff(2).max_delay(ms(5));

                run(&script, &chain(vec![Arc::new(retry)]), task("x")).unwrap();

                assert_eq!(script.sleeps(), [ms(2), ms(4), ms(5)]);
            }
        }

        mod retry_after {
            use super::*;

            fn asking(after: u64) -> Done {
                Done {
                    retry_after: Some(ms(after)),
                    ..done(1)
                }
            }

            fn retry(retries: u32) -> Retry<Task> {
                Retry::new(retries, ms(1)).retry_after(|output: &Done| output.retry_after)
            }

            #[test]
            fn retry_after_should_wait_as_long_as_output_asks() {
                let script = Script::new([Ok(asking(30)), Ok(asking(0))]);

                run(&script, &chain(vec![Arc::new(retry(2))]), task("x")).unwrap();

                assert_eq!(script.sleeps(), [ms(30), ms(1)]);
            }

            #[test]
            fn retry_after_longer_than_max_delay_should_not_retry() {
                let script = Script::new([Ok(asking(30))]);
                let retry = retry(1).max_delay(ms(10));

                run(&script, &chain(vec![Arc::new(retry)]), task("x")).unwrap();

                assert_eq!(script.calls().len(), 1);
            }

            #[test]
            fn retry_after_equal_to_max_delay_should_retry() {
                let script = Script::new([Ok(asking(10))]);
                let retry = retry(1).max_delay(ms(10));

                run(&script, &chain(vec![Arc::new(retry)]), task("x")).unwrap();

                assert_eq!(script.sleeps(), [ms(10)]);
            }

            #[test]
            fn retry_after_error_should_wait_delay() {
                let script = Script::new([Err(crate::Error::process("io".to_string()))]);

                run(&script, &chain(vec![Arc::new(retry(1))]), task("x")).unwrap();

                assert_eq!(script.sleeps(), [ms(1)]);
            }
        }

        mod deadline {
            use super::*;

            #[test]
            fn deadline_should_stop_before_wait_ending_past_it() {
                let script = Script::new([Ok(done(1)), Ok(done(1)), Ok(done(1)), Ok(done(1))]);

                let output = run(
                    &script,
                    &chain(vec![Arc::new(
                        Retry::new(u32::MAX, ms(10)).backoff(2).deadline(ms(35)),
                    )]),
                    task("x"),
                )
                .unwrap();

                assert_eq!(output, done(1));
                assert_eq!(script.calls().len(), 3);
                assert_eq!(script.sleeps(), [ms(10), ms(20)]);
            }

            #[test]
            fn deadline_should_allow_wait_ending_at_it() {
                let script = Script::new([Ok(done(1)), Ok(done(1))]);

                run(
                    &script,
                    &chain(vec![Arc::new(
                        Retry::new(u32::MAX, ms(10)).deadline(ms(20)),
                    )]),
                    task("x"),
                )
                .unwrap();

                assert_eq!(script.sleeps(), [ms(10), ms(10)]);
            }

            #[test]
            fn deadline_should_stop_before_asked_wait_ending_past_it() {
                let asking = Done {
                    status: 1,
                    retry_after: Some(ms(30)),
                    ..Default::default()
                };
                let script = Script::new([Ok(asking.clone())]);

                let output = run(
                    &script,
                    &chain(vec![Arc::new(
                        Retry::new(1, ms(1))
                            .retry_after(|output: &Done| output.retry_after)
                            .deadline(ms(20)),
                    )]),
                    task("x"),
                )
                .unwrap();

                assert_eq!(output, asking);
                assert!(script.sleeps().is_empty());
            }

            #[test]
            fn deadline_should_count_wall_clock_elapsed() {
                let retry = Retry::<Task>::new(1, ms(1)).deadline(ms(5));
                let mut state = RetryState::new();
                std::thread::sleep(ms(10));

                assert_eq!(retry.next_delay(&task("x"), &Ok(done(1)), &mut state), None);
            }
        }

        mod next_delay {
            use super::*;

            #[test]
            fn next_delay_should_record_retries_in_state() {
                let retry = Retry::<Task>::new(2, ms(1)).backoff(3);
                let mut state = RetryState::new();
                let failed = Ok(done(1));

                let delays = [(); 3].map(|()| retry.next_delay(&task("x"), &failed, &mut state));

                assert_eq!(delays, [Some(ms(1)), Some(ms(3)), None]);
                assert_eq!(state.retried(), 2);
            }

            #[test]
            fn next_delay_success_should_return_none() {
                let retry = Retry::<Task>::new(2, ms(1));
                let mut state = RetryState::new();

                let succeeded = Ok(Done {
                    success: true,
                    ..done(0)
                });

                assert_eq!(retry.next_delay(&task("x"), &succeeded, &mut state), None);
                assert_eq!(state.retried(), 0);
            }
        }

        mod when {
            use super::*;

            #[test]
            fn when_should_decide_instead_of_retrying_all_failures() {
                let script = Script::new([Ok(done(3)), Ok(done(1))]);
                let retry = Retry::<Task>::new(3, ms(1))
                    .when(|_, result| result.as_ref().is_ok_and(|output| output.status == 3));

                let output = run(&script, &chain(vec![Arc::new(retry)]), task("x")).unwrap();

                assert_eq!(output, done(1));
                assert_eq!(script.calls().len(), 2);
            }

            #[test]
            fn when_should_not_retry_success() {
                let script = Script::default();
                let retry = Retry::<Task>::new(3, ms(1)).when(|_, _| true);

                run(&script, &chain(vec![Arc::new(retry)]), task("x")).unwrap();

                assert_eq!(script.calls().len(), 1);
            }
        }

        mod clone {
            use super::*;

            #[test]
            fn clone_should_copy_configuration() {
                let retry = Retry::<Task>::new(2, ms(1))
                    .backoff(2)
                    .deadline(ms(5))
                    .when(|_, _| true)
                    .retry_after(|_| None);

                assert_eq!(format!("{:?}", retry.clone()), format!("{retry:?}"));
            }
        }

        mod std_fmt_debug {
            use super::*;

            #[test]
            fn fmt_should_show_configuration() {
                let retry = Retry::<Task>::new(2, ms(1));

                assert_eq!(
                    format!("{retry:?}"),
                    "Retry { retries: 2, delay: 1ms, backoff: 1, max_delay: 60s, deadline: None, condition: false, retry_after: false }"
                );
            }
        }

        mod interceptor {
            use super::*;

            mod handle {
                use super::*;

                #[test]
                fn handle_nested_retry_should_be_retried_as_a_whole() {
                    let script = Script::new([Ok(done(1)), Ok(done(1)), Ok(done(1)), Ok(done(1))]);
                    let interceptors = chain(vec![
                        Arc::new(Retry::new(1, ms(1))),
                        Arc::new(Retry::new(1, ms(2))),
                    ]);

                    let output = run(&script, &interceptors, task("x")).unwrap();

                    assert_eq!(output, done(1));
                    assert_eq!(script.calls().len(), 4);
                    assert_eq!(script.sleeps(), [ms(2), ms(1), ms(2)]);
                }

                #[test]
                fn handle_after_sink_accepted_output_should_not_retry() {
                    let script = Script::new([Ok(data(1, "out"))]);

                    run_to(
                        &script,
                        &chain(vec![Arc::new(Retry::new(1, ms(1)))]),
                        task("x"),
                        &mut Vec::new(),
                    )
                    .unwrap();

                    assert_eq!(script.calls().len(), 1);
                }

                #[test]
                fn handle_replayed_run_should_return_replay_output_once() {
                    crate::interceptor::tests::interceptor::handle::handle_replayed_run_should_return_replay_output_once(
                        Arc::new(Retry::new(1, ms(1))),
                        &Script::default(),
                        task("x").with_replay_output(done(1)),
                    );
                }

                #[test]
                fn handle_invalid_run_should_fail_in_one_attempt() {
                    let script = Script::default();
                    let mut task = task("x");
                    task.subject_mut().invalid = true;

                    crate::interceptor::tests::interceptor::handle::handle_invalid_run_should_fail_in_one_attempt(
                        Arc::new(Retry::new(2, ms(1))),
                        &script,
                        task,
                    );

                    assert!(script.sleeps().is_empty());
                }

                #[test]
                fn handle_failing_sink_should_not_retry() {
                    let script = Script::new([Ok(data(0, "out"))]);
                    let mut sink = |_: u8, _: &[u8]| Err(crate::Error::process("closed"));

                    chain(vec![Arc::new(Retry::new(2, ms(1)))])
                        .run_blocking(&script, &mut task("x"), Some(&mut sink))
                        .unwrap_err();

                    assert_eq!(script.calls().len(), 1);
                }

                #[test]
                fn handle_chunk_rejected_by_interceptor_should_not_retry() {
                    let script = Script::new([Ok(data(0, "out"))]);

                    run(
                        &script,
                        &chain(vec![Arc::new(Retry::new(2, ms(1))), Arc::new(Reject)]),
                        task("x"),
                    )
                    .unwrap_err();

                    assert_eq!(script.calls().len(), 1);
                }

                #[test]
                fn handle_should_retry_executor_error_after_rejected_attempt() {
                    let script = Script::new([Err(crate::Error::process("io")), Ok(done(0))]);
                    let interceptors = chain(vec![Arc::new(Retry::new(2, ms(1))), Arc::new(Fix)]);
                    let mut task = task("x");
                    task.subject_mut().invalid = true;

                    let output = run(&script, &interceptors, task).unwrap();

                    assert!(output.success);
                    assert_eq!(script.calls().len(), 2);
                }

                /// Rejects every chunk.
                struct Reject;

                impl Interceptor<Task> for Reject {
                    fn chunk(&self, _: Chunk<Task>) -> Result {
                        Err(crate::Error::process("rejected"))
                    }
                }

                /// Makes a rejected run valid and attempts it again.
                struct Fix;

                impl Interceptor<Task> for Fix {
                    fn handle<'a>(
                        &'a self,
                        mut next: Next<'a, Task>,
                    ) -> BoxFuture<'a, Result<Done>> {
                        Box::pin(async move {
                            match next.attempt().await {
                                Err(_) if next.run().rejected() => {
                                    next.run_mut().subject_mut().invalid = false;
                                    next.attempt().await
                                }
                                result => result,
                            }
                        })
                    }
                }

                #[test]
                fn handle_should_pass_chunks_on() {
                    let mut sunk = Vec::new();

                    run_to(
                        &Script::new([Ok(data(0, "out"))]),
                        &chain(vec![Arc::new(Retry::new(1, ms(1)))]),
                        task("x"),
                        &mut sunk,
                    )
                    .unwrap();

                    assert_eq!(sunk, [(1, b"out".to_vec())]);
                }
            }

            mod chunk {
                use super::*;

                #[test]
                fn chunk_should_pass_chunk_on_unchanged() {
                    crate::interceptor::tests::interceptor::chunk::chunk_should_pass_chunk_on_unchanged(
                        Arc::new(Retry::<Task>::new(1, ms(1))),
                        &task("x"),
                        1,
                    );
                }
            }
        }
    }

    mod retry_state {
        use super::*;

        mod retried {
            use super::*;

            #[test]
            fn retried_new_state_should_return_zero() {
                assert_eq!(RetryState::new().retried(), 0);
            }
        }

        mod default {
            use super::*;

            #[test]
            fn default_should_return_new_state() {
                assert_eq!(RetryState::default().retried(), 0);
            }
        }
    }
}
