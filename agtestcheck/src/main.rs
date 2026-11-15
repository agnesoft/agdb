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
    run(&platform(), &mut std::io::stdout())
}

fn run(p: &impl Platform, out: &mut dyn Write) -> ExitCode {
    let issues = check(&source_map(p, &sources(p)));

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
        use agplatform::Fs;

        #[test]
        fn run_without_issues_should_succeed_and_print_nothing() {
            let platform = test_platform();
            platform.fs().create_dir_all("src").unwrap();
            platform
                .fs()
                .write(
                    "src/lib.rs",
                    "fn f() {}\n#[cfg(test)]\nmod tests {\n    mod f {\n        #[test]\n        fn f_should_work() {}\n    }\n}\n",
                )
                .unwrap();
            let mut out = Vec::new();

            assert_eq!(run(&platform, &mut out), ExitCode::SUCCESS);
            assert_eq!(String::from_utf8(out).unwrap(), "");
        }

        #[test]
        fn run_with_issues_should_fail_and_print_them() {
            let platform = test_platform();
            platform
                .fs()
                .write(
                    "lib.rs",
                    "#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n}\n",
                )
                .unwrap();
            let mut out = Vec::new();

            assert_eq!(run(&platform, &mut out), ExitCode::FAILURE);
            assert_eq!(
                String::from_utf8(out).unwrap(),
                "lib.rs:4: test `t` should be in the module of its type, trait, function or method\n",
            );
        }
    }
}
