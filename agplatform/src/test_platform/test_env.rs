use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use crate::Env;
use crate::EnvVars;
use crate::env::EnvArgs;
use crate::env::EnvImpl;
use crate::test_platform::ROOT;

/// Enabled by the `testing` feature flag.
///
/// A memory-only [`crate::Env`] implementation for tests.
/// Use [`Self::new`] together with [`Self::with_args`],
/// [`Self::with_vars`], and [`Self::with_current_dir`] to seed
/// arguments, variables, and the current directory before passing it
/// to code that expects an environment implementation.
///
/// This is a lightweight wrapper around the internal `EnvImpl`.
///
/// # Examples
///
/// ```rust
/// use agplatform::Env;
/// use agplatform::TestEnv;
///
/// let env = TestEnv::new()
///     .with_args(vec!["app".to_string(), "--verbose".to_string()])
///     .with_vars(vec![("TEST_VAR".to_string(), "value".to_string())]);
///
/// assert_eq!(env.args().collect::<Vec<_>>(), ["app", "--verbose"]);
/// assert_eq!(env.var("TEST_VAR"), Some("value".into()));
/// ```
pub struct TestEnv(EnvImpl);

impl TestEnv {
    /// Creates a new empty test environment whose current directory is the
    /// [`ROOT`] directory.
    ///
    /// This is the test-only counterpart to `EnvImpl::new`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::TestEnv;
    ///
    /// let env = TestEnv::new();
    /// ```
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(EnvImpl::new(Vec::new(), PathBuf::from(ROOT)))
    }

    /// Sets the command-line arguments used by this test environment.
    ///
    /// See [`crate::Env::args`] for how the values are read back.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Env;
    /// use agplatform::TestEnv;
    ///
    /// let env = TestEnv::new().with_args(vec!["app", "--help"]);
    /// assert_eq!(env.args().collect::<Vec<_>>(), ["app", "--help"]);
    /// ```
    pub fn with_args<T: Into<OsString>>(mut self, args: Vec<T>) -> Self {
        self.0
            .replace_args(args.into_iter().map(Into::into).collect());
        self
    }

    /// Sets the current directory used by this test environment.
    ///
    /// See [`crate::Env::current_dir`] for the read-side API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Env;
    /// use agplatform::TestEnv;
    /// use std::path::PathBuf;
    ///
    /// let env = TestEnv::new().with_current_dir(PathBuf::from("/tmp/project"));
    /// assert_eq!(env.current_dir(), std::path::Path::new("/tmp/project"));
    /// ```
    pub fn with_current_dir<P: Into<PathBuf>>(self, current_dir: P) -> Self {
        self.0.replace_current_dir(current_dir.into());
        self
    }

    /// Sets the current executable used by this test environment.
    ///
    /// See [`crate::Env::current_exe`] for the read-side API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Env;
    /// use agplatform::TestEnv;
    /// use std::path::PathBuf;
    ///
    /// let env = TestEnv::new().with_current_exe(PathBuf::from("/tmp/project/app"));
    /// assert_eq!(env.current_exe(), std::path::Path::new("/tmp/project/app"));
    /// ```
    pub fn with_current_exe<P: Into<PathBuf>>(mut self, current_exe: P) -> Self {
        self.0.replace_current_exe(current_exe.into());
        self
    }

    /// Sets the environment variables used by this test environment.
    ///
    /// See [`crate::Env::vars`] and [`crate::Env::var`] for the read-side APIs.
    ///
    /// # Panics
    ///
    /// In debug builds, if a variable would be rejected by
    /// [`crate::Env::set_var`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Env;
    /// use agplatform::TestEnv;
    ///
    /// let env = TestEnv::new().with_vars(vec![("TEST_VAR", "value")]);
    /// assert_eq!(env.var("TEST_VAR"), Some("value".into()));
    /// ```
    pub fn with_vars<K: Into<OsString>, V: Into<OsString>>(self, vars: Vec<(K, V)>) -> Self {
        self.0.replace_vars(
            vars.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        );
        self
    }

    /// [internal]
    ///
    /// Returns a handle to this environment that sees its changes.
    #[cfg(any(feature = "fs", feature = "process", feature = "request"))]
    pub(crate) fn share(&self) -> EnvImpl {
        self.0.clone()
    }
}

impl Env for TestEnv {
    /// Returns the command-line arguments stored in the test environment.
    ///
    /// See [`crate::Env::args`].
    fn args(&self) -> EnvArgs<'_> {
        self.0.args()
    }

    /// Returns the current directory stored in the test environment.
    ///
    /// See [`crate::Env::current_dir`].
    fn current_dir(&self) -> PathBuf {
        self.0.current_dir()
    }

    /// Returns the current executable stored in the test environment.
    ///
    /// See [`crate::Env::current_exe`].
    fn current_exe(&self) -> &Path {
        self.0.current_exe()
    }

    /// Returns the home directory stored in the test environment.
    ///
    /// See [`crate::Env::home_dir`].
    fn home_dir(&self) -> Option<PathBuf> {
        self.0.home_dir()
    }

    /// Removes an environment variable from the test environment.
    ///
    /// See [`crate::Env::remove_var`].
    fn remove_var<K: AsRef<OsStr>>(&self, key: K) -> Option<OsString> {
        self.0.remove_var(key)
    }

    /// Sets the current directory in the test environment, without
    /// checking that it exists.
    ///
    /// See [`crate::Env::set_current_dir`].
    fn set_current_dir<P: AsRef<Path>>(&self, path: P) -> crate::Result<PathBuf> {
        self.0.set_current_dir_checked(path.as_ref(), &|_| Ok(()))
    }

    /// Sets an environment variable in the test environment.
    ///
    /// See [`crate::Env::set_var`].
    fn set_var<K: Into<OsString>, V: Into<OsString>>(
        &self,
        key: K,
        value: V,
    ) -> crate::Result<Option<OsString>> {
        self.0.set_var(key, value)
    }

    /// Returns the temporary directory of the test environment.
    ///
    /// See [`crate::Env::tmp_dir`].
    fn tmp_dir(&self) -> PathBuf {
        self.0.tmp_dir()
    }

    /// Returns an environment variable from the test environment.
    ///
    /// See [`crate::Env::var`].
    fn var<K: AsRef<OsStr>>(&self, key: K) -> Option<OsString> {
        self.0.var(key)
    }

    /// Returns the environment variables stored in the test environment.
    ///
    /// See [`crate::Env::vars`].
    fn vars(&self) -> EnvVars {
        self.0.vars()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_env {
        use super::*;

        mod env {
            use super::*;

            fn dir() -> PathBuf {
                Path::new(ROOT).join("home")
            }

            mod args {
                use super::*;

                #[test]
                fn args_should_return_arguments_in_order() {
                    let env = TestEnv::new().with_args(vec!["app", "--flag"]);

                    crate::env::tests::env::args::args_should_return_arguments_in_order(
                        &env,
                        &["app", "--flag"],
                    );
                }
            }

            mod current_dir {
                use super::*;

                #[test]
                fn current_dir_should_return_current_directory() {
                    crate::env::tests::env::current_dir::current_dir_should_return_current_directory(
                        &TestEnv::new(),
                        Path::new(ROOT),
                    );
                }
            }

            mod current_exe {
                use super::*;

                #[test]
                fn current_exe_should_return_current_executable() {
                    let env = TestEnv::new().with_current_exe("/app");

                    crate::env::tests::env::current_exe::current_exe_should_return_current_executable(
                        &env,
                        Path::new("/app"),
                    );
                }
            }

            mod home_dir {
                use super::*;

                #[test]
                fn home_dir_should_return_home_variable() {
                    crate::env::tests::env::home_dir::home_dir_should_return_home_variable(
                        &TestEnv::new(),
                    );
                }

                #[test]
                fn home_dir_unset_or_empty_variable_should_return_none() {
                    crate::env::tests::env::home_dir::home_dir_unset_or_empty_variable_should_return_none(&TestEnv::new());
                }
            }

            mod remove_var {
                use super::*;

                #[test]
                fn remove_var_should_return_value_for_present_key() {
                    crate::env::tests::env::remove_var::remove_var_should_return_value_for_present_key(&TestEnv::new());
                }

                #[test]
                fn remove_var_should_return_none_for_missing_key() {
                    crate::env::tests::env::remove_var::remove_var_should_return_none_for_missing_key(&TestEnv::new());
                }

                #[cfg(windows)]
                #[test]
                fn remove_var_should_match_key_ignoring_ascii_case() {
                    crate::env::tests::env::remove_var::remove_var_should_match_key_ignoring_ascii_case(&TestEnv::new());
                }

                #[cfg(unix)]
                #[test]
                fn remove_var_should_match_key_case_sensitively() {
                    crate::env::tests::env::remove_var::remove_var_should_match_key_case_sensitively(&TestEnv::new());
                }
            }

            mod set_current_dir {
                use super::*;

                #[test]
                fn set_current_dir_should_return_previous_value() {
                    crate::env::tests::env::set_current_dir::set_current_dir_should_return_previous_value(
                        &TestEnv::new(),
                        &dir(),
                    );
                }

                #[test]
                fn set_current_dir_should_update_current_directory() {
                    crate::env::tests::env::set_current_dir::set_current_dir_should_update_current_directory(
                        &TestEnv::new(),
                        &dir(),
                    );
                }

                #[test]
                fn set_current_dir_relative_should_resolve_against_previous_current_dir() {
                    crate::env::tests::env::set_current_dir::set_current_dir_relative_should_resolve_against_previous_current_dir(
                        &TestEnv::new(),
                        &dir(),
                    );
                }

                #[test]
                fn set_current_dir_should_not_go_above_root() {
                    crate::env::tests::env::set_current_dir::set_current_dir_should_not_go_above_root(
                        &TestEnv::new(),
                        &dir(),
                    );
                }

                #[test]
                fn set_current_dir_missing_directory_should_set_it() {
                    let env = TestEnv::new();

                    env.set_current_dir("missing").unwrap();

                    assert_eq!(env.current_dir(), Path::new(ROOT).join("missing"));
                }
            }

            mod set_var {
                use super::*;

                #[test]
                fn set_var_should_return_none_for_new_key() {
                    crate::env::tests::env::set_var::set_var_should_return_none_for_new_key(
                        &TestEnv::new(),
                    );
                }

                #[test]
                fn set_var_should_return_previous_value_for_existing_key() {
                    crate::env::tests::env::set_var::set_var_should_return_previous_value_for_existing_key(&TestEnv::new());
                }

                #[test]
                fn set_var_invalid_key_should_fail_and_keep_vars() {
                    crate::env::tests::env::set_var::set_var_invalid_key_should_fail_and_keep_vars(
                        &TestEnv::new(),
                    );
                }

                #[test]
                fn set_var_value_with_nul_should_fail_and_keep_vars() {
                    crate::env::tests::env::set_var::set_var_value_with_nul_should_fail_and_keep_vars(&TestEnv::new());
                }

                #[cfg(windows)]
                #[test]
                fn set_var_different_case_should_replace_value_keeping_key() {
                    crate::env::tests::env::set_var::set_var_different_case_should_replace_value_keeping_key(&TestEnv::new());
                }

                #[cfg(unix)]
                #[test]
                fn set_var_different_case_should_add_variable() {
                    crate::env::tests::env::set_var::set_var_different_case_should_add_variable(
                        &TestEnv::new(),
                    );
                }
            }

            mod tmp_dir {
                use super::*;

                #[cfg(unix)]
                #[test]
                fn tmp_dir_should_return_tmpdir_variable() {
                    crate::env::tests::env::tmp_dir::tmp_dir_should_return_tmpdir_variable(
                        &TestEnv::new(),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn tmp_dir_unset_variable_should_return_default() {
                    crate::env::tests::env::tmp_dir::tmp_dir_unset_variable_should_return_default(
                        &TestEnv::new(),
                    );
                }

                #[cfg(windows)]
                #[test]
                fn tmp_dir_should_return_first_set_variable_with_separator() {
                    crate::env::tests::env::tmp_dir::tmp_dir_should_return_first_set_variable_with_separator(&TestEnv::new());
                }
            }

            mod var {
                use super::*;

                #[test]
                fn var_should_return_none_for_missing_key() {
                    crate::env::tests::env::var::var_should_return_none_for_missing_key(
                        &TestEnv::new(),
                    );
                }

                #[test]
                fn var_should_return_value_for_present_key() {
                    crate::env::tests::env::var::var_should_return_value_for_present_key(
                        &TestEnv::new(),
                    );
                }

                #[test]
                fn var_should_return_value_that_is_not_utf8() {
                    crate::env::tests::env::var::var_should_return_value_that_is_not_utf8(
                        &TestEnv::new(),
                    );
                }

                #[cfg(windows)]
                #[test]
                fn var_should_match_key_ignoring_ascii_case() {
                    crate::env::tests::env::var::var_should_match_key_ignoring_ascii_case(
                        &TestEnv::new(),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn var_should_match_key_case_sensitively() {
                    crate::env::tests::env::var::var_should_match_key_case_sensitively(
                        &TestEnv::new(),
                    );
                }
            }

            mod var_bool {
                use super::*;

                #[test]
                fn var_bool_should_accept_truthy_values() {
                    crate::env::tests::env::var_bool::var_bool_should_accept_truthy_values(
                        &TestEnv::new(),
                    );
                }

                #[test]
                fn var_bool_should_accept_falsy_values() {
                    crate::env::tests::env::var_bool::var_bool_should_accept_falsy_values(
                        &TestEnv::new(),
                    );
                }

                #[test]
                fn var_bool_should_trim_value_before_unquoting() {
                    crate::env::tests::env::var_bool::var_bool_should_trim_value_before_unquoting(
                        &TestEnv::new(),
                    );
                }

                #[test]
                fn var_bool_should_reject_invalid_values() {
                    crate::env::tests::env::var_bool::var_bool_should_reject_invalid_values(
                        &TestEnv::new(),
                    );
                }

                #[test]
                fn var_bool_should_return_false_for_missing_key() {
                    crate::env::tests::env::var_bool::var_bool_should_return_false_for_missing_key(
                        &TestEnv::new(),
                    );
                }
            }

            mod vars {
                use super::*;

                #[test]
                fn vars_should_include_set_variable() {
                    crate::env::tests::env::vars::vars_should_include_set_variable(&TestEnv::new());
                }

                #[test]
                fn vars_should_exclude_removed_variable() {
                    crate::env::tests::env::vars::vars_should_exclude_removed_variable(
                        &TestEnv::new(),
                    );
                }

                #[test]
                fn vars_should_return_snapshot() {
                    crate::env::tests::env::vars::vars_should_return_snapshot(&TestEnv::new());
                }
            }
        }
    }
}
