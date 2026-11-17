mod check;
mod source_map;
mod sources;

use std::io::Write;
use std::process::ExitCode;

use agplatform::Platform;
use agplatform::platform;

use crate::check::check;
use crate::source_map::source_map;
use crate::sources::sources;

fn main() -> ExitCode {
    run(&platform(), &mut std::io::stdout(), &mut std::io::stderr())
}

/// The issues go to `out`, the problems that keep sources from being
/// checked to `err`.
fn run(p: &impl Platform, out: &mut dyn Write, err: &mut dyn Write) -> ExitCode {
    let map = source_map(p, &sources(p));
    let issues = check(&map);

    for warning in &map.warnings {
        writeln!(err, "{warning}").expect("writing the warning should succeed");
    }

    for issue in &issues {
        writeln!(
            out,
            "{}:{}: {}",
            issue.file.display(),
            issue.line,
            issue.kind
        )
        .expect("writing the issue should succeed");
    }

    if issues.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::test_platform::test_platform;

    mod main {
        use super::*;

        #[test]
        fn main_should_check_current_dir() {
            let _ = main();
        }
    }

    mod run {
        use super::*;
        use crate::source_map::tests::cargo_metadata;
        use agplatform::Fs;

        #[test]
        fn run_without_issues_should_succeed_and_print_nothing() {
            let platform = test_platform();
            platform.fs().create_dir_all("src").unwrap();
            platform.fs().write("Cargo.toml", "").unwrap();
            cargo_metadata(&platform, &[("", &[("lib", "src/lib.rs")])]);
            platform
                .fs()
                .write(
                    "src/lib.rs",
                    "fn f() {}\n#[cfg(test)]\nmod tests {\n    mod f {\n        #[test]\n        fn f_should_work() {}\n    }\n}\n",
                )
                .unwrap();
            let mut out = Vec::new();
            let mut err = Vec::new();

            assert_eq!(run(&platform, &mut out, &mut err), ExitCode::SUCCESS);
            assert_eq!(String::from_utf8(out).unwrap(), "");
            assert_eq!(String::from_utf8(err).unwrap(), "");
        }

        #[test]
        fn run_with_issues_should_fail_and_print_them() {
            let platform = test_platform();
            platform.fs().write("Cargo.toml", "").unwrap();
            cargo_metadata(&platform, &[("", &[("lib", "lib.rs")])]);
            platform
                .fs()
                .write(
                    "lib.rs",
                    "#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n",
                )
                .unwrap();
            let mut out = Vec::new();
            let mut err = Vec::new();

            assert_eq!(run(&platform, &mut out, &mut err), ExitCode::FAILURE);
            assert_eq!(
                String::from_utf8(out).unwrap(),
                "lib.rs:4: test `t` should be in the module of its type, trait, function or method\n",
            );
            assert_eq!(String::from_utf8(err).unwrap(), "");
        }

        #[test]
        fn run_with_unreached_sources_should_succeed_and_warn_of_them() {
            let platform = test_platform();
            platform
                .fs()
                .write("loose.rs", "#[test]\nfn t() {}\n")
                .unwrap();
            let mut out = Vec::new();
            let mut err = Vec::new();

            assert_eq!(run(&platform, &mut out, &mut err), ExitCode::SUCCESS);
            assert_eq!(String::from_utf8(out).unwrap(), "");
            assert_eq!(
                String::from_utf8(err).unwrap(),
                "loose.rs: not reachable from any crate root\n"
            );
        }
    }
}
