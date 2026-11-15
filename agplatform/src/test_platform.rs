#[cfg(any(feature = "process", feature = "request"))]
mod gate;
pub mod test_env;
#[cfg(feature = "fs")]
pub mod test_fs;
#[cfg(feature = "process")]
pub mod test_process;
#[cfg(feature = "request")]
pub mod test_request;

use crate::Platform;
use crate::TestEnv;

/// The root directory of a new [`TestEnv`], `TestFs` and `TestProcess`.
#[cfg(unix)]
pub const ROOT: &str = "/";
/// The root directory of a new [`TestEnv`], `TestFs` and `TestProcess`.
#[cfg(windows)]
pub const ROOT: &str = "C:\\";

/// Enabled by the `testing` feature flag.
///
/// The test platform is a memory-only but fully functional mock
/// implementation of the [`Platform`] trait. It is used in tests to be
/// passed where `impl Platform` is expected.
///
/// # Examples
///
/// ```rust
/// use agplatform::Platform;
/// use agplatform::test_platform::test_platform;
///
/// fn do_something(platform: &impl Platform) {}
///
/// let test_platform = test_platform();
/// do_something(&test_platform);
/// ```
pub struct TestPlatform {
    pub env: TestEnv,
    #[cfg(feature = "fs")]
    pub fs: crate::TestFs,
    #[cfg(feature = "process")]
    pub process: crate::TestProcess,
    #[cfg(feature = "request")]
    pub request: crate::TestRequest,
}

impl TestPlatform {
    /// Creates a test platform with the specified test environment.
    ///
    /// Like in production, its current directory becomes that of the file
    /// system and process interfaces and its variables configure the
    /// request interface. Its variables are the environment inherited by
    /// the commands of the process interface, so they are not recorded as
    /// changes in `TestCall::vars`.
    ///
    /// See [`crate::TestEnv`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Platform, Env};
    /// use agplatform::test_platform::test_platform;
    ///
    /// let test_env = agplatform::TestEnv::new().with_args(vec!["arg1", "arg2"]);
    /// let test_platform = test_platform().with_env(test_env);
    /// assert_eq!(test_platform.env().args().collect::<Vec<_>>(), &["arg1", "arg2"]);
    /// ```
    pub fn with_env(mut self, env: TestEnv) -> Self {
        #[cfg(feature = "fs")]
        self.fs.set_env(env.share());
        #[cfg(feature = "process")]
        self.process.set_env(env.share());
        #[cfg(feature = "request")]
        self.request.set_env(env.share());
        self.env = env;
        self
    }

    /// Enabled by the `fs` feature flag.
    ///
    /// Creates a test platform with the specified test file system, whose
    /// current directory is set to that of the environment.
    ///
    /// See [`crate::TestFs`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Platform, Fs};
    /// use agplatform::test_platform::test_platform;
    ///
    /// let test_fs = agplatform::TestFs::new();
    /// let test_platform = test_platform().with_fs(test_fs);
    /// ```
    #[cfg(feature = "fs")]
    pub fn with_fs(mut self, mut fs: crate::TestFs) -> Self {
        fs.set_env(self.env.share());
        self.fs = fs;
        self
    }

    /// Enabled by the `process` feature flag.
    ///
    /// Creates a test platform with the specified test process interface,
    /// whose current directory is set to that of the environment.
    ///
    /// See [`crate::TestProcess`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Platform;
    /// use agplatform::test_platform::test_platform;
    ///
    /// let test_process = agplatform::TestProcess::new();
    /// let test_platform = test_platform().with_process(test_process);
    /// ```
    #[cfg(feature = "process")]
    pub fn with_process(mut self, mut process: crate::TestProcess) -> Self {
        process.set_env(self.env.share());
        self.process = process;
        self
    }

    /// Enabled by the `request` feature flag.
    ///
    /// Creates a test platform with the specified test request interface,
    /// configured by the variables of the environment.
    ///
    /// See [`crate::TestRequest`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Platform;
    /// use agplatform::test_platform::test_platform;
    ///
    /// let test_request = agplatform::TestRequest::new();
    /// let test_platform = test_platform().with_request(test_request);
    /// ```
    #[cfg(feature = "request")]
    pub fn with_request(mut self, mut request: crate::TestRequest) -> Self {
        request.set_env(self.env.share());
        self.request = request;
        self
    }
}

impl Platform for TestPlatform {
    type Env = TestEnv;
    #[cfg(feature = "fs")]
    type Fs = crate::TestFs;
    #[cfg(feature = "process")]
    type Process = crate::TestProcess;
    #[cfg(feature = "request")]
    type Request = crate::TestRequest;

    /// Returns a reference to the test environment.
    ///
    /// See [`crate::Platform::env`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Env, Platform};
    /// use agplatform::test_platform::test_platform;
    ///
    /// let platform = test_platform();
    /// let env = platform.env();
    /// ```
    fn env(&self) -> &TestEnv {
        &self.env
    }

    /// Returns a reference to the test file system.
    ///
    /// See [`crate::Platform::fs`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    /// use agplatform::test_platform::test_platform;
    ///
    /// let platform = test_platform();
    /// let fs = platform.fs();
    /// ```
    #[cfg(feature = "fs")]
    fn fs(&self) -> &crate::TestFs {
        &self.fs
    }

    /// Returns a reference to the test process interface.
    ///
    /// See [`crate::Platform::process`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Platform, Process};
    /// use agplatform::test_platform::test_platform;
    ///
    /// let platform = test_platform();
    /// let process = platform.process();
    /// ```
    #[cfg(feature = "process")]
    fn process(&self) -> &crate::TestProcess {
        &self.process
    }

    /// Returns a reference to the test request interface.
    ///
    /// See [`crate::Platform::request`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Platform, Request};
    /// use agplatform::test_platform::test_platform;
    ///
    /// let platform = test_platform();
    /// let request = platform.request();
    /// ```
    #[cfg(feature = "request")]
    fn request(&self) -> &crate::TestRequest {
        &self.request
    }
}

/// Returns an instance of [`TestPlatform`] that implements the
/// [`Platform`] trait via the mock implementations, sharing its
/// [`TestEnv`].
///
/// # Examples
///
/// ```rust
/// use agplatform::Platform;
/// use agplatform::test_platform::test_platform;
///
/// fn do_something(platform: &impl Platform) {}
///
/// let test_platform = test_platform();
/// do_something(&test_platform);
/// ```
pub fn test_platform() -> TestPlatform {
    TestPlatform {
        env: TestEnv::new(),
        #[cfg(feature = "fs")]
        fs: crate::TestFs::new(),
        #[cfg(feature = "process")]
        process: crate::TestProcess::new(),
        #[cfg(feature = "request")]
        request: crate::TestRequest::new(),
    }
    .with_env(TestEnv::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Env;

    #[cfg(unix)]
    const HOME: &str = "/home";
    #[cfg(windows)]
    const HOME: &str = "C:\\home";

    #[cfg(feature = "request")]
    fn reqwest_env(https_proxy: &str) -> crate::request::ReqwestEnv {
        crate::request::ReqwestEnv::new(&[("HTTPS_PROXY".into(), https_proxy.into())])
    }

    mod test_platform {
        use super::*;

        mod with_env {
            use super::*;

            #[test]
            fn with_env_should_set_custom_test_environment() {
                let test_env = TestEnv::new()
                    .with_args(vec!["arg1", "arg2"])
                    .with_current_dir(HOME)
                    .with_current_exe("/tmp/project/app")
                    .with_vars(vec![("KEY1", "value1"), ("KEY2", "value2")]);
                let p = test_platform().with_env(test_env);
                let e = p.env();

                assert_eq!(e.args().collect::<Vec<_>>(), &["arg1", "arg2"]);
                assert_eq!(e.current_dir(), std::path::Path::new(HOME));
                assert_eq!(e.set_current_dir("..").unwrap(), std::path::Path::new(HOME));
                assert_eq!(e.current_dir(), std::path::Path::new(ROOT));
                assert_eq!(e.current_exe(), std::path::Path::new("/tmp/project/app"));
                assert_eq!(e.home_dir(), None);
                assert_eq!(
                    e.tmp_dir(),
                    crate::env::EnvImpl::new(Vec::new(), std::path::PathBuf::new()).tmp_dir()
                );
                assert_eq!(e.var("KEY1"), Some("value1".into()));
                assert_eq!(e.var("KEY2"), Some("value2".into()));
                assert_eq!(e.var("NON_EXISTENT_KEY"), None);
                assert_eq!(e.set_var("NON_EXISTENT_KEY", "value").unwrap(), None);
                assert_eq!(e.remove_var("NON_EXISTENT_KEY"), Some("value".into()));
                assert_eq!(
                    e.set_var("KEY1", "new_value").unwrap(),
                    Some("value1".into())
                );
                assert_eq!(
                    e.vars().collect::<Vec<_>>(),
                    vec![
                        ("KEY1".into(), "new_value".into()),
                        ("KEY2".into(), "value2".into())
                    ]
                );
            }

            #[cfg(feature = "request")]
            #[test]
            fn with_env_should_pass_proxy_variables_to_request() {
                let test_env = TestEnv::new().with_vars(vec![("HTTPS_PROXY", "http://proxy")]);

                let test_platform = test_platform().with_env(test_env);

                assert_eq!(
                    test_platform.request.clients.reqwest_env(),
                    reqwest_env("http://proxy")
                );
            }

            #[cfg(feature = "fs")]
            #[test]
            fn with_env_should_set_current_dir_of_fs() {
                use crate::Fs;

                // GIVEN a platform with a file system
                let test_platform = test_platform().with_fs(crate::TestFs::new());

                // WHEN an environment with another current directory is set
                let test_platform = test_platform.with_env(TestEnv::new().with_current_dir(HOME));

                // THEN relative fs paths resolve against it
                test_platform.fs().create_dir_all(HOME).unwrap();
                assert_eq!(
                    test_platform.fs().canonicalize(".").unwrap(),
                    std::path::Path::new(HOME)
                );
            }

            #[cfg(feature = "process")]
            #[test]
            fn with_env_should_set_current_dir_of_process() {
                let test_platform = test_platform()
                    .with_process(crate::TestProcess::new())
                    .with_env(TestEnv::new().with_current_dir(HOME));

                assert_eq!(
                    test_platform.process.current_dir(),
                    std::path::Path::new(HOME)
                );
            }

            #[cfg(feature = "process")]
            #[test]
            fn with_env_should_pass_vars_to_commands() {
                use crate::CommandExt;

                // GIVEN a platform with an environment holding a variable
                let test_platform =
                    test_platform().with_env(TestEnv::new().with_vars(vec![("KEY", "value")]));
                test_platform.process.set_output(
                    "git",
                    crate::CommandOutput {
                        status: Some(0),
                        ..Default::default()
                    },
                );

                // WHEN running a command
                std::process::Command::new("git")
                    .run_blocking(test_platform.process())
                    .unwrap();

                // THEN the command gets the variable
                assert_eq!(
                    test_platform.process.call_details()[0].vars,
                    [("KEY".into(), "value".into())]
                );
            }
        }

        #[cfg(feature = "process")]
        mod with_process {
            use super::*;

            #[test]
            fn with_process_should_take_current_dir_of_env() {
                let test_platform = test_platform()
                    .with_env(TestEnv::new().with_current_dir(HOME))
                    .with_process(crate::TestProcess::new());

                assert_eq!(
                    test_platform.process.current_dir(),
                    std::path::Path::new(HOME)
                );
            }
        }

        #[cfg(feature = "request")]
        mod with_request {
            use super::*;

            #[test]
            fn with_request_should_pass_proxy_variables_of_env() {
                let test_env = TestEnv::new().with_vars(vec![("HTTPS_PROXY", "http://proxy")]);

                let test_platform = test_platform()
                    .with_env(test_env)
                    .with_request(crate::TestRequest::new());

                assert_eq!(
                    test_platform.request.clients.reqwest_env(),
                    reqwest_env("http://proxy")
                );
            }
        }

        #[cfg(feature = "fs")]
        mod with_fs {
            use super::*;
            use crate::Fs;

            #[test]
            fn with_fs_should_set_custom_test_filesystem() {
                let test_fs = crate::TestFs::new();
                let test_platform = test_platform().with_fs(test_fs);
                test_platform.fs().create_dir_all("/tmp").unwrap();
                test_platform.env().set_current_dir("/tmp").unwrap();
                test_platform.fs().write("file.txt", "hello").unwrap();

                assert_eq!(
                    test_platform.fs().read("/tmp/file.txt").unwrap(),
                    vec![b'h', b'e', b'l', b'l', b'o'],
                );
                assert_eq!(
                    test_platform.fs().read_to_string("/tmp/file.txt").unwrap(),
                    "hello",
                );
            }

            #[test]
            fn with_fs_should_take_current_dir_of_env() {
                // GIVEN a platform with an environment with a current directory
                let test_platform = test_platform().with_env(TestEnv::new().with_current_dir(HOME));

                // WHEN a file system is set
                let test_platform = test_platform.with_fs(crate::TestFs::new());

                // THEN relative fs paths resolve against the current directory
                test_platform.fs().create_dir_all(HOME).unwrap();
                assert_eq!(
                    test_platform.fs().canonicalize(".").unwrap(),
                    std::path::Path::new(HOME)
                );
            }
        }

        mod platform {
            use super::*;

            mod env {
                use super::*;

                #[test]
                fn env_should_keep_changes_across_calls() {
                    crate::platform::tests::platform::env::env_should_keep_changes_across_calls(
                        &test_platform(),
                    );
                }

                #[cfg(all(feature = "fs", feature = "process"))]
                #[test]
                fn env_set_current_dir_should_update_fs_and_process() {
                    use crate::Fs;

                    // GIVEN an existing directory
                    let test_platform = test_platform();
                    test_platform.fs().create_dir_all("/tmp").unwrap();

                    #[cfg(unix)]
                    const CURRENT_DIR: &str = "/tmp";
                    #[cfg(windows)]
                    const CURRENT_DIR: &str = "C:\\tmp";

                    // WHEN it becomes the current directory
                    test_platform.env().set_current_dir(CURRENT_DIR).unwrap();

                    // THEN relative fs paths resolve against it
                    assert_eq!(
                        test_platform.fs().canonicalize(".").unwrap(),
                        std::path::Path::new(CURRENT_DIR)
                    );
                    assert_eq!(
                        test_platform.env.current_dir(),
                        std::path::Path::new(CURRENT_DIR)
                    );
                    assert_eq!(
                        test_platform.process.current_dir(),
                        std::path::Path::new(CURRENT_DIR)
                    );
                }

                #[cfg(all(feature = "fs", feature = "process"))]
                #[test]
                fn env_set_current_dir_relative_should_pass_resolved_dir_to_fs_and_process() {
                    use crate::Fs;

                    // GIVEN a platform in an existing directory
                    let test_platform =
                        test_platform().with_env(TestEnv::new().with_current_dir(HOME));
                    test_platform.fs().create_dir_all(HOME).unwrap();
                    let expected = std::path::Path::new(HOME).join("a");
                    test_platform.fs().create_dir_all("a").unwrap();

                    // WHEN a relative directory becomes the current directory
                    test_platform.env().set_current_dir("b/../a/.").unwrap();

                    // THEN the env, fs and process resolve it against the previous one
                    assert_eq!(test_platform.env.current_dir(), expected);
                    assert_eq!(test_platform.fs().canonicalize(".").unwrap(), expected);
                    assert_eq!(test_platform.process.current_dir(), expected);
                }

                #[cfg(feature = "process")]
                #[test]
                fn env_set_and_remove_var_should_be_applied_to_commands() {
                    use crate::CommandExt;

                    // GIVEN a platform with variables set and removed
                    let test_platform = test_platform();
                    test_platform.process.set_output(
                        "git",
                        crate::CommandOutput {
                            status: Some(0),
                            ..Default::default()
                        },
                    );
                    test_platform.env().set_var("REMOVED", "value").unwrap();
                    test_platform.env().set_var("SET", "value").unwrap();
                    test_platform.env().remove_var("REMOVED");

                    // WHEN running a command
                    std::process::Command::new("git")
                        .run_blocking(test_platform.process())
                        .unwrap();

                    // THEN it gets the variables as changed
                    assert_eq!(
                        test_platform.process.call_details()[0].vars,
                        [("SET".into(), "value".into())]
                    );
                }

                #[cfg(feature = "request")]
                #[test]
                fn env_set_and_remove_var_should_update_request_proxy_variables() {
                    let test_platform = test_platform();

                    test_platform
                        .env()
                        .set_var("HTTPS_PROXY", "http://proxy")
                        .unwrap();
                    let set = test_platform.request.clients.reqwest_env();
                    test_platform.env().remove_var("HTTPS_PROXY");

                    assert_eq!(set, reqwest_env("http://proxy"));
                    assert_eq!(
                        test_platform.request.clients.reqwest_env(),
                        crate::request::ReqwestEnv::default()
                    );
                }

                #[cfg(feature = "fs")]
                #[test]
                fn env_set_current_dir_should_be_seen_by_platform_shared_in_arc() {
                    use crate::Fs;
                    use std::sync::Arc;

                    // GIVEN a platform shared with another thread
                    let platform =
                        Arc::new(test_platform().with_env(TestEnv::new().with_current_dir(HOME)));
                    platform.fs().create_dir_all("a").unwrap();
                    let shared = platform.clone();

                    // WHEN the other thread changes the current directory
                    std::thread::spawn(move || shared.env().set_current_dir("a"))
                        .join()
                        .unwrap()
                        .unwrap();

                    // THEN relative paths resolve against it
                    assert_eq!(
                        platform.fs().canonicalize(".").unwrap(),
                        std::path::Path::new(HOME).join("a")
                    );
                }
            }

            #[cfg(feature = "fs")]
            mod fs {
                use super::*;

                #[test]
                fn fs_should_resolve_relative_paths_against_current_dir_of_env() {
                    use crate::Fs;

                    let test_platform = test_platform();
                    test_platform.fs().create_dir_all(HOME).unwrap();

                    crate::platform::tests::platform::fs::fs_should_resolve_relative_paths_against_current_dir_of_env(
                        &test_platform,
                        std::path::Path::new(HOME),
                    );
                }
            }

            #[cfg(feature = "process")]
            mod process {
                use super::*;

                #[test]
                fn process_should_run_commands_through_its_interceptors() {
                    crate::platform::tests::platform::process::process_should_run_commands_through_its_interceptors(
                        &test_platform(),
                    );
                }
            }

            #[cfg(feature = "request")]
            mod request {
                use super::*;

                #[test]
                fn request_should_send_requests_through_its_interceptors() {
                    crate::platform::tests::platform::request::request_should_send_requests_through_its_interceptors(
                        &test_platform(),
                    );
                }
            }
        }
    }
}
