use std::ffi::OsString;

use crate::interceptor::Cache;

use super::ProcessCommand;

/// The [`Cache`] of the runs of commands by their
/// [`ProcessCommand::cache_key`]. The outputs of
/// [`CommandOptions::interactive`](crate::CommandOptions::interactive) runs are
/// not cached, nor those the [`Cache`] does not cache, e.g. passed to a sink.
///
/// The variables of the platform are not part of the key unless named
/// with [`with_vars`](Cache::with_vars), so that unrelated ones, e.g. the
/// id of a CI job, do not change it.
///
/// # Examples
///
/// ```rust
/// use agplatform::{CommandCache, CommandExt, CommandOutput, Process, TestProcess};
/// use std::process::Command;
///
/// let cache = CommandCache::default();
/// let process = TestProcess::new();
/// process.add_interceptor(cache.clone());
/// process.set_output("git", CommandOutput { stdout: b"abc".to_vec(), ..Default::default() });
///
/// Command::new("git").args(["rev-parse", "HEAD"]).run_blocking(&process).unwrap();
/// let output = Command::new("git").args(["rev-parse", "HEAD"]).run_blocking(&process).unwrap();
///
/// assert_eq!(output.stdout, b"abc");
/// assert_eq!(process.calls().len(), 1);
/// assert_eq!(cache.entries().len(), 1);
/// ```
pub type CommandCache = Cache<ProcessCommand>;

impl Cache<ProcessCommand> {
    /// Creates the cache of the runs of commands by their
    /// [`ProcessCommand::cache_key`] with the platform variables `names`,
    /// e.g. those configuring the commands like `KUBECONFIG`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{CommandCache, CommandExt, CommandOutput, Env, Platform, Process};
    /// use std::process::Command;
    ///
    /// let cache = CommandCache::with_vars(["KUBECONFIG"]);
    /// let platform = agplatform::test_platform::test_platform();
    /// platform.process.add_interceptor(cache.clone());
    /// platform.process.set_output("kubectl", CommandOutput::default());
    ///
    /// for config in ["a", "b", "a"] {
    ///     platform.env().set_var("KUBECONFIG", config).unwrap();
    ///     Command::new("kubectl").arg("get").run_blocking(platform.process()).unwrap();
    /// }
    ///
    /// assert_eq!(platform.process.calls().len(), 2);
    /// ```
    pub fn with_vars<K: Into<OsString>>(names: impl IntoIterator<Item = K>) -> Self {
        let vars: Vec<OsString> = names.into_iter().map(Into::into).collect();
        Self::new(move |run| run.subject().cache_key(&vars))
    }
}

/// The cache of the runs of commands by their [`ProcessCommand::cache_key`]
/// without platform variables.
impl Default for Cache<ProcessCommand> {
    fn default() -> Self {
        Self::new(|run| run.subject().cache_key(&[]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TestDir;
    use crate::interceptor::Interceptors;
    use crate::process::CommandExt;
    use crate::process::CommandOutput;
    use crate::process::Process;
    use crate::process::ProcessImpl;
    use crate::process::tests::*;
    use std::time::Duration;

    mod cache {
        use super::*;

        /// Returns a process caching in `cache` the runs in `dir`.
        fn cached(dir: &TestDir, cache: &CommandCache) -> ProcessImpl {
            let process = process(dir.path().to_path_buf());
            process.add_interceptor(cache.clone());
            process
        }

        #[test]
        fn cache_should_return_cached_output_of_same_command() {
            // GIVEN a command run once through the cache
            let dir = TestDir::new();
            let cache = CommandCache::default();
            let process = cached(&dir, &cache);
            let first = process
                .run_blocking(&shell("echo out&& touch marker"))
                .unwrap();
            std::fs::remove_file(dir.path().join("marker")).unwrap();

            // WHEN running it again
            let second = process
                .run_blocking(&shell("echo out&& touch marker"))
                .unwrap();

            // THEN its cached output is returned without running it
            assert_eq!(second, first);
            assert!(!dir.path().join("marker").exists());
        }

        #[test]
        fn cache_should_not_cache_failed_replayed_sunk_or_interactive_outputs() {
            // GIVEN a cache after an interceptor replaying an output
            let dir = TestDir::new();
            let cache = CommandCache::default();
            let process = cached(&dir, &cache);
            let mut sunk = (Vec::new(), Vec::new());
            let mut replayed = Interceptors::new();
            replayed.push(Replay(exit(0)));
            replayed.push(cache.clone());

            // WHEN running a failing, a replayed, a sunk and an interactive
            // command
            let _ = process.run_blocking(&shell("exit 1"));
            shell("exit 0")
                .interceptors(replayed)
                .run_blocking(&process)
                .unwrap();
            process
                .run_blocking_with_sink(&shell("echo out"), &mut sink_into(&mut sunk))
                .unwrap();
            shell("exit 0")
                .interactive()
                .run_blocking(&process)
                .unwrap();

            // THEN none is cached
            assert!(cache.entries().is_empty());
        }

        #[test]
        fn cache_loaded_should_return_output_instead_of_running() {
            // GIVEN a cache loaded with the output of a command
            let dir = TestDir::new();
            let cache = CommandCache::default();
            let process = cached(&dir, &cache);
            let command = shell("touch marker");
            let key = cache.key(&resolved(&command, &process)).unwrap();
            cache.load([(key, exit(0))]);

            // WHEN running it
            let output = process.run_blocking(&command).unwrap();

            // THEN the loaded output is returned without running it
            assert_eq!(output, exit(0));
            assert!(!dir.path().join("marker").exists());
        }

        #[test]
        fn cache_loaded_longer_than_timeout_should_return_output() {
            let dir = TestDir::new();
            let cache = CommandCache::default();
            let process = cached(&dir, &cache);
            let command = shell("exit 1");
            let slow = CommandOutput {
                duration: Duration::from_secs(3),
                ..exit(0)
            };
            let mut run = resolved(&command, &process);
            run.subject_mut().options_mut().timeout = Some(Duration::from_secs(1));
            cache.load([(cache.key(&run).unwrap(), slow.clone())]);

            let output = command
                .timeout(Duration::from_secs(1))
                .run_blocking(&process);

            assert_eq!(output.unwrap(), slow);
        }

        #[cfg(feature = "tokio")]
        #[test]
        fn cache_async_should_return_cached_output_of_same_command() {
            let dir = TestDir::new();
            let cache = CommandCache::default();
            let process = cached(&dir, &cache);
            let first = block_on(process.run_async(&shell("echo out"))).unwrap();

            let second = block_on(process.run_async(&shell("echo out"))).unwrap();

            assert_eq!(second, first);
            assert_eq!(cache.entries().len(), 1);
        }

        mod with_vars {
            use super::*;

            #[test]
            fn with_vars_should_key_runs_by_named_platform_vars() {
                // GIVEN a cache keyed by a platform variable
                let dir = TestDir::new();
                let cache = CommandCache::with_vars(["AGPLATFORM_CONFIG"]);
                let process = cached(&dir, &cache);

                // WHEN running a command with its values changed in between
                for config in ["a", "b", "a"] {
                    process.set_var("AGPLATFORM_CONFIG".to_string(), config.to_string());
                    process
                        .run_blocking(&print_var("AGPLATFORM_CONFIG"))
                        .unwrap();
                }

                // THEN the command is cached by each value
                let entries = cache.entries();
                assert_eq!(entries.len(), 2);
                assert!(
                    entries[0]
                        .0
                        .contains("vars: [(\"AGPLATFORM_CONFIG\", \"set\")]"),
                    "{entries:?}"
                );
            }
        }
    }

    mod default {
        use super::*;

        #[allow(clippy::module_inception)]
        mod default {
            use super::*;

            #[test]
            fn default_should_key_runs_without_platform_vars() {
                let dir = TestDir::new();
                let cache = CommandCache::default();
                let process = process(dir.path().to_path_buf());
                process.add_interceptor(cache.clone());

                for config in ["a", "b"] {
                    process.set_var("AGPLATFORM_CONFIG".to_string(), config.to_string());
                    process
                        .run_blocking(&print_var("AGPLATFORM_CONFIG"))
                        .unwrap();
                }

                let entries = cache.entries();
                assert_eq!(entries.len(), 1);
                assert!(entries[0].0.contains("vars: []"), "{entries:?}");
            }
        }
    }
}
