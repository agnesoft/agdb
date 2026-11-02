//! Configuration of a command line tool read from its arguments and
//! environment.
//!
//! The tool reads them through `&impl Platform` instead of `std::env`, so
//! its tests give it a `TestEnv` with exactly the arguments, variables and
//! directories they need. They never touch the environment of the test
//! process, which is shared by all tests running in parallel.
//!
//! ```bash
//! cargo run --example env --features env -- --verbose
//! ```

use std::path::PathBuf;

use agplatform::Env;
use agplatform::Platform;

#[derive(Debug, PartialEq)]
struct Config {
    config_dir: PathBuf,
    verbose: bool,
    files: Vec<PathBuf>,
}

fn config(platform: &impl Platform) -> agplatform::Result<Config> {
    let env = platform.env();
    let config_dir = match env.var("XDG_CONFIG_HOME") {
        Some(dir) if !dir.is_empty() => PathBuf::from(dir),
        _ => env
            .home_dir()
            .ok_or_else(|| agplatform::Error::env("Neither XDG_CONFIG_HOME nor HOME is set"))?
            .join(".config"),
    };
    // WHY: the arguments are not converted to strings, so that file names
    // that are not valid UTF-8 are kept as they are.
    let (flags, files): (Vec<_>, Vec<_>) = env
        .args()
        .skip(1)
        .partition(|arg| arg.as_encoded_bytes().starts_with(b"--"));

    Ok(Config {
        config_dir: config_dir.join("tool"),
        verbose: flags.iter().any(|flag| *flag == "--verbose") || env.var_bool("TOOL_VERBOSE")?,
        files: files.into_iter().map(PathBuf::from).collect(),
    })
}

/// Makes `workspace` the current directory of the tool and exports it to
/// the processes it starts. Returns the previous current directory.
///
/// Fails, exporting nothing, if `workspace` is not an existing directory.
fn enter_workspace(platform: &impl Platform, workspace: PathBuf) -> agplatform::Result<PathBuf> {
    let env = platform.env();
    let previous = env.set_current_dir(&workspace)?;
    env.set_var("TOOL_WORKSPACE", workspace)?;
    Ok(previous)
}

fn main() -> agplatform::Result<()> {
    let platform = agplatform::platform();
    let config = config(&platform)?;
    println!("{config:?}");

    let workspace = platform.env().tmp_dir();
    let previous = enter_workspace(&platform, workspace)?;
    println!(
        "moved from {} to {}",
        previous.display(),
        platform.env().current_dir().display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::TestEnv;
    use agplatform::test_platform::test_platform;

    #[cfg(unix)]
    const HOME_VAR: &str = "HOME";
    #[cfg(windows)]
    const HOME_VAR: &str = "USERPROFILE";

    mod config {
        use super::*;

        #[test]
        fn config_should_split_arguments_into_flags_and_files() {
            let env = TestEnv::new()
                .with_args(vec!["tool", "a.txt", "--verbose", "b.txt"])
                .with_vars(vec![(HOME_VAR, "/home")]);
            let platform = test_platform().with_env(env);

            let config = config(&platform).unwrap();

            assert!(config.verbose);
            assert_eq!(
                config.files,
                [PathBuf::from("a.txt"), PathBuf::from("b.txt")]
            );
        }

        #[test]
        fn config_should_prefer_xdg_config_home_over_home_dir() {
            for (vars, expected) in [
                (vec![], "/home/user/.config/tool"),
                (vec![("XDG_CONFIG_HOME", "")], "/home/user/.config/tool"),
                (vec![("XDG_CONFIG_HOME", "/xdg")], "/xdg/tool"),
            ] {
                let env = TestEnv::new()
                    .with_vars([vec![(HOME_VAR, "/home/user")], vars.clone()].concat());
                let platform = test_platform().with_env(env);

                let config = config(&platform).unwrap();

                assert_eq!(config.config_dir, PathBuf::from(expected), "vars: {vars:?}");
            }
        }

        #[test]
        fn config_without_home_should_fail() {
            let platform = test_platform();

            let error = config(&platform).unwrap_err();

            assert_eq!(error.kind(), agplatform::ErrorKind::Env);
        }

        #[test]
        fn config_should_read_verbose_from_variable() {
            for (value, expected) in [("1", true), ("on", true), ("false", false)] {
                let env =
                    TestEnv::new().with_vars(vec![(HOME_VAR, "/home"), ("TOOL_VERBOSE", value)]);
                let platform = test_platform().with_env(env);

                let config = config(&platform).unwrap();

                assert_eq!(config.verbose, expected, "TOOL_VERBOSE: {value}");
            }
        }

        #[test]
        fn config_invalid_verbose_variable_should_fail() {
            let env =
                TestEnv::new().with_vars(vec![(HOME_VAR, "/home"), ("TOOL_VERBOSE", "maybe")]);
            let platform = test_platform().with_env(env);

            let error = config(&platform).unwrap_err();

            assert_eq!(error.kind(), agplatform::ErrorKind::Env);
            assert!(error.description().contains("TOOL_VERBOSE"));
        }
    }

    mod enter_workspace {
        use super::*;
        use agplatform::test_platform::ROOT;

        #[test]
        fn enter_workspace_should_change_current_dir_and_export_it() {
            let platform = test_platform().with_env(TestEnv::new().with_current_dir("/home"));

            let previous = enter_workspace(&platform, PathBuf::from(ROOT)).unwrap();

            assert_eq!(previous, PathBuf::from("/home"));
            assert_eq!(platform.env.current_dir(), PathBuf::from(ROOT));
            assert_eq!(platform.env.var("TOOL_WORKSPACE"), Some(ROOT.into()));
        }

        #[test]
        fn enter_workspace_path_with_nul_should_fail() {
            // GIVEN a test env, which does not check that the directory exists
            let platform = test_platform();

            let error = enter_workspace(&platform, PathBuf::from("/a\0b")).unwrap_err();

            assert_eq!(error.kind(), agplatform::ErrorKind::Env);
            assert_eq!(platform.env.var("TOOL_WORKSPACE"), None);
        }
    }
}
