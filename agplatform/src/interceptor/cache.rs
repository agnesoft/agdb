use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;

use super::{BoxFuture, Interceptor, Next, Outcome, Run, Subject};
use crate::Result;

/// The [`Interceptor`] caching the successful outputs of the runs in memory
/// by their [`key`](Self::key), returning them for the runs with the same
/// key instead of running them again. Runs without a key, the outputs set
/// by other interceptors (see [`Run::is_replay`]), those returned by the
/// interceptors after it without an attempt reaching the executor, and
/// those passed to a sink are not cached.
///
/// A hit is replayed, so its data, already changed by the chunk
/// interceptors when it ran, passes through them again, see
/// [`Interceptor::chunk`]. Concurrent runs with the same key that miss all
/// run, each caching its output.
///
/// Clones share the cache, e.g. to persist its [`entries`](Self::entries)
/// and [`load`](Self::load) them later.
pub struct Cache<S: Subject> {
    outputs: Arc<Mutex<BTreeMap<String, S::Output>>>,
    key: Arc<CacheKey<S>>,
}

type CacheKey<S> = dyn Fn(&Run<S>) -> Option<String> + Send + Sync;

impl<S: Subject> Cache<S> {
    /// Creates the cache of the runs by their `key`, `None` for a run that
    /// is not cached.
    pub fn new<F>(key: F) -> Self
    where
        F: Fn(&Run<S>) -> Option<String> + Send + Sync + 'static,
    {
        Self {
            outputs: Arc::default(),
            key: Arc::new(key),
        }
    }

    /// Returns the cached outputs by their keys, e.g. to persist them.
    pub fn entries(&self) -> Vec<(String, S::Output)> {
        self.outputs()
            .iter()
            .map(|(key, output)| (key.clone(), output.clone()))
            .collect()
    }

    /// Returns the key of `run`, `None` if it is not cached.
    pub fn key(&self, run: &Run<S>) -> Option<String> {
        (self.key)(run)
    }

    /// Adds the `entries`, e.g. persisted from [`Self::entries`], replacing
    /// those with the same keys.
    pub fn load(&self, entries: impl IntoIterator<Item = (String, S::Output)>) {
        self.outputs().extend(entries);
    }

    fn outputs(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, S::Output>> {
        self.outputs.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Sets the cached output of the run of `next` as its replay output, if
    /// any, unless the run is replayed already.
    fn hit(&self, next: &mut Next<S>, key: &str) {
        let output = self.outputs().get(key).cloned();

        if !next.run().is_replay()
            && let Some(output) = output
        {
            next.set_replay_output(output);
        }
    }

    /// Caches the successful output of `run` from `result` by `key`, if it
    /// ran after `attempt`.
    fn store(&self, run: &Run<S>, attempt: u32, key: String, result: &Result<S::Output>) {
        if let Ok(output) = result
            && output.success()
            && run.attempt() > attempt
            && !run.is_replay()
            && !run.has_sink()
        {
            self.outputs().insert(key, output.clone());
        }
    }
}

impl<S: Subject> Clone for Cache<S> {
    fn clone(&self) -> Self {
        Self {
            outputs: self.outputs.clone(),
            key: self.key.clone(),
        }
    }
}

impl<S: Subject> std::fmt::Debug for Cache<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Cache({})", self.outputs().len())
    }
}

impl<S: Subject> Interceptor<S> for Cache<S> {
    fn handle<'a>(&'a self, mut next: Next<'a, S>) -> BoxFuture<'a, Result<S::Output>> {
        Box::pin(async move {
            let Some(key) = self.key(next.run()) else {
                return next.attempt().await;
            };

            self.hit(&mut next, &key);
            let attempt = next.run().attempt();
            let result = next.attempt().await;
            self.store(next.run(), attempt, key, &result);
            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interceptor::DryRun;
    use crate::interceptor::tests::*;

    mod cache {
        use super::*;

        fn named() -> Cache<Task> {
            Cache::<Task>::new(|run| {
                Some(run.subject().name.clone()).filter(|name| name != "uncached")
            })
        }

        #[test]
        fn cache_should_return_cached_output_instead_of_running() {
            let (cache, script) = (
                named(),
                Script::new([Ok(data(0, "first")), Ok(data(0, "second"))]),
            );
            let interceptors = chain(vec![Arc::new(cache.clone())]);

            run(&script, &interceptors, task("x")).unwrap();
            let output = run(&script, &interceptors, task("x")).unwrap();

            assert_eq!(output.data, b"first");
            assert_eq!(script.calls(), ["x"]);
        }

        #[test]
        fn cache_hit_should_pass_data_through_chunk_interceptors_again() {
            let (cache, events) = (named(), Events::default());
            let interceptors = chain(vec![
                Arc::new(cache.clone()),
                Arc::new(log("a", &events)),
                Arc::new(Redact(Some("***"))),
            ]);

            let ran = run(
                &Script::new([Ok(data(0, "secret"))]),
                &interceptors,
                task("x"),
            )
            .unwrap();
            let hit = run(&Script::default(), &interceptors, task("x")).unwrap();

            assert_eq!(ran, hit);
            assert_eq!(hit.data, b"***");
            assert_eq!(
                recorded(&events)
                    .iter()
                    .filter(|event| event.starts_with("a chunk 1"))
                    .count(),
                2
            );
        }

        #[test]
        fn cache_hit_with_sink_should_pass_data_to_sink() {
            let cache = named();
            cache.load([("x".to_string(), data(0, "cached"))]);
            let mut sunk = Vec::new();

            let output = run_to(
                &Script::default(),
                &chain(vec![Arc::new(cache)]),
                task("x"),
                &mut sunk,
            )
            .unwrap();

            assert_eq!(sunk, [(1, b"cached".to_vec())]);
            assert!(output.success);
        }

        mod new {
            use super::*;

            #[test]
            fn new_run_without_key_should_not_be_cached() {
                let (cache, script) = (named(), Script::default());
                let interceptors = chain(vec![Arc::new(cache.clone())]);

                run(&script, &interceptors, task("uncached")).unwrap();
                run(&script, &interceptors, task("uncached")).unwrap();

                assert_eq!(script.calls().len(), 2);
                assert!(cache.entries().is_empty());
            }
        }

        mod entries {
            use super::*;

            #[test]
            fn entries_should_return_cached_outputs_by_key() {
                let cache = named();

                run(
                    &Script::new([Ok(data(0, "out"))]),
                    &chain(vec![Arc::new(cache.clone())]),
                    task("x"),
                )
                .unwrap();

                assert_eq!(
                    cache.entries(),
                    [(
                        "x".to_string(),
                        Done {
                            success: true,
                            ..data(0, "out")
                        }
                    )]
                );
            }
        }

        mod key {
            use super::*;

            #[test]
            fn key_should_return_key_of_run() {
                assert_eq!(named().key(&task("x")), Some("x".to_string()));
                assert_eq!(named().key(&task("uncached")), None);
            }
        }

        mod load {
            use super::*;

            #[test]
            fn load_should_add_entries() {
                let (cache, script) = (named(), Script::default());
                cache.load([("x".to_string(), data(0, "loaded"))]);

                let output = run(&script, &chain(vec![Arc::new(cache)]), task("x")).unwrap();

                assert_eq!(output.data, b"loaded");
                assert!(script.calls().is_empty());
            }

            #[test]
            fn load_should_replace_entries_with_same_key() {
                let cache = named();
                cache.load([
                    ("x".to_string(), data(0, "old")),
                    ("y".to_string(), done(0)),
                ]);

                cache.load([("x".to_string(), data(0, "new"))]);

                assert_eq!(
                    cache.entries(),
                    [
                        ("x".to_string(), data(0, "new")),
                        ("y".to_string(), done(0))
                    ]
                );
            }
        }

        mod clone {
            use super::*;

            #[test]
            fn clone_should_share_cache() {
                let cache = named();

                cache.clone().load([("x".to_string(), done(0))]);

                assert_eq!(cache.entries().len(), 1);
            }
        }

        mod std_fmt_debug {
            use super::*;

            #[test]
            fn fmt_should_show_number_of_entries() {
                let cache = named();
                cache.load([("x".to_string(), done(0))]);

                assert_eq!(format!("{cache:?}"), "Cache(1)");
            }
        }

        mod interceptor {
            use super::*;

            mod handle {
                use super::*;

                #[test]
                fn handle_should_not_cache_failure() {
                    let cache = named();

                    run(
                        &Script::new([Ok(done(1))]),
                        &chain(vec![Arc::new(cache.clone())]),
                        task("x"),
                    )
                    .unwrap();

                    assert!(cache.entries().is_empty());
                }

                #[test]
                fn handle_should_not_cache_error() {
                    let cache = named();

                    run(
                        &Script::new([Err(crate::Error::process("io".to_string()))]),
                        &chain(vec![Arc::new(cache.clone())]),
                        task("x"),
                    )
                    .unwrap_err();

                    assert!(cache.entries().is_empty());
                }

                #[test]
                fn handle_should_cache_success_after_retried_failure() {
                    for retry_outside in [false, true] {
                        let (cache, retry) =
                            (named(), Arc::new(crate::interceptor::Retry::new(1, ms(1))));
                        let interceptors = if retry_outside {
                            chain(vec![retry, Arc::new(cache.clone())])
                        } else {
                            chain(vec![Arc::new(cache.clone()), retry])
                        };

                        run(
                            &Script::new([Ok(done(1)), Ok(data(0, "retried"))]),
                            &interceptors,
                            task("x"),
                        )
                        .unwrap();

                        assert_eq!(
                            cache.entries(),
                            [(
                                "x".to_string(),
                                Done {
                                    success: true,
                                    ..data(0, "retried")
                                }
                            )],
                            "retry outside: {retry_outside}"
                        );
                    }
                }

                #[test]
                fn handle_should_not_cache_output_passed_to_sink() {
                    let cache = named();

                    run_to(
                        &Script::default(),
                        &chain(vec![Arc::new(cache.clone())]),
                        task("x"),
                        &mut Vec::new(),
                    )
                    .unwrap();

                    assert!(cache.entries().is_empty());
                }

                #[test]
                fn handle_should_not_cache_output_set_by_other_interceptor() {
                    let cache = named();

                    run(
                        &Script::default(),
                        &chain(vec![Arc::new(cache.clone()), Arc::new(DryRun)]),
                        task("x"),
                    )
                    .unwrap();

                    assert!(cache.entries().is_empty());
                }

                #[test]
                fn handle_should_not_cache_output_returned_without_attempt() {
                    let cache = named();

                    run(
                        &Script::default(),
                        &chain(vec![Arc::new(cache.clone()), Arc::new(Stub)]),
                        task("x"),
                    )
                    .unwrap();

                    assert!(cache.entries().is_empty());
                }

                /// Returns a success without an attempt.
                struct Stub;

                impl Interceptor<Task> for Stub {
                    fn handle<'a>(&'a self, _: Next<'a, Task>) -> BoxFuture<'a, Result<Done>> {
                        Box::pin(std::future::ready(Ok(Done {
                            success: true,
                            ..done(0)
                        })))
                    }
                }

                #[test]
                fn handle_replayed_run_should_return_replay_output_once() {
                    let cache = named();
                    cache.load([("x".to_string(), data(0, "cached"))]);

                    crate::interceptor::tests::interceptor::handle::handle_replayed_run_should_return_replay_output_once(
                        Arc::new(cache.clone()),
                        &Script::default(),
                        task("x").with_replay_output(data(0, "set")),
                    );

                    assert_eq!(cache.entries(), [("x".to_string(), data(0, "cached"))]);
                }

                #[test]
                fn handle_invalid_run_should_fail_in_one_attempt() {
                    let cache = named();
                    let mut task = task("x");
                    task.subject_mut().invalid = true;

                    crate::interceptor::tests::interceptor::handle::handle_invalid_run_should_fail_in_one_attempt(
                        Arc::new(cache.clone()),
                        &Script::default(),
                        task,
                    );

                    assert!(cache.entries().is_empty());
                }
            }

            mod chunk {
                use super::*;

                #[test]
                fn chunk_should_pass_chunk_on_unchanged() {
                    crate::interceptor::tests::interceptor::chunk::chunk_should_pass_chunk_on_unchanged(
                        Arc::new(named()),
                        &task("x"),
                        1,
                    );
                }
            }
        }
    }
}
