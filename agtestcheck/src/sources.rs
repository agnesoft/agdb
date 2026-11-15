use std::path::Path;
use std::path::PathBuf;

use agplatform::DirEntry;
use agplatform::Env;
use agplatform::Fs;
use agplatform::Platform;

const CACHEDIR_SIGNATURE: &str = "Signature: 8a477f597d28d172789f06886806bc55";

pub(crate) fn sources(p: &impl Platform) -> Vec<PathBuf> {
    let mut sources = Vec::new();
    let mut args = p.env().args().skip(1).peekable();
    args.next_if(|arg| *arg == "test-check");

    if let Some(path) = args.next() {
        if is_rust_source(path) {
            if p.fs().open_read(path).is_ok() {
                sources.push(path.into());
            }
        } else {
            sources = read_dir_sources(Path::new(path), p);
        }
    } else {
        sources = read_dir_sources(Path::new("."), p);
    }

    let current_dir = p.env().current_dir();

    sources
        .into_iter()
        .map(|source| {
            source
                .strip_prefix(&current_dir)
                .map(Path::to_path_buf)
                .unwrap_or(source)
        })
        .collect()
}

pub(crate) fn warn_unreadable(error: &agplatform::Error) {
    eprintln!("cannot read: {error}");
}

fn read_dir_sources(path: &Path, p: &impl Platform) -> Vec<PathBuf> {
    match p.fs().read_dir(path) {
        Ok(mut dir) => read_sources_in(&mut dir, p),
        Err(error) => {
            warn_unreadable(&error);
            Vec::new()
        }
    }
}

fn read_sources_in(
    dir: &mut dyn Iterator<Item = agplatform::Result<DirEntry>>,
    p: &impl Platform,
) -> Vec<PathBuf> {
    let mut sources = Vec::new();

    for entry in dir {
        match entry {
            Ok(entry) => sources.extend(read_dir_entry(entry, p)),
            Err(error) => warn_unreadable(&error),
        }
    }

    sources
}

fn read_dir_entry(entry: DirEntry, p: &impl Platform) -> Vec<PathBuf> {
    match entry.file_type() {
        agplatform::FileType::File if is_rust_source(entry.path()) => {
            vec![PathBuf::from(entry.path())]
        }
        agplatform::FileType::Dir if is_cache_dir(&entry, p) => Vec::new(),
        agplatform::FileType::Dir => read_dir_sources(entry.path(), p),
        _ => Vec::new(),
    }
}

/// Cargo tags its target directories, whatever their name, with a
/// `CACHEDIR.TAG` (https://bford.info/cachedir/).
fn is_cache_dir(entry: &DirEntry, p: &impl Platform) -> bool {
    p.fs()
        .read_to_string(entry.path().join("CACHEDIR.TAG"))
        .is_ok_and(|tag| tag.starts_with(CACHEDIR_SIGNATURE))
}

fn is_rust_source<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().extension().and_then(|s| s.to_str()) == Some("rs")
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::test_platform::test_platform;

    mod sources {
        use super::*;
        use agplatform::TestEnv;

        #[test]
        fn sources_no_args_should_return_all_sources() {
            let platform = test_platform();
            platform.fs().create_dir_all("src/check").unwrap();
            platform.fs().write("src/main.rs", b"fn main() {}").unwrap();
            platform.fs().write("src/check/check.rs", b"").unwrap();

            let mut result = sources(&platform);
            result.sort();
            assert_eq!(
                result,
                vec![
                    PathBuf::from("src/check/check.rs"),
                    PathBuf::from("src/main.rs"),
                ]
            );
        }

        #[test]
        fn sources_with_arg_should_return_specific_source() {
            let platform =
                test_platform().with_env(TestEnv::new().with_args(vec!["program", "src/main.rs"]));
            platform.fs().create_dir_all("src/check").unwrap();
            platform.fs().write("src/main.rs", b"fn main() {}").unwrap();
            platform.fs().write("src/check/check.rs", b"").unwrap();

            let result = sources(&platform);
            assert_eq!(result, vec![PathBuf::from("src/main.rs")]);
        }

        #[test]
        fn sources_as_cargo_subcommand_should_return_all_sources() {
            let platform =
                test_platform().with_env(TestEnv::new().with_args(vec!["program", "test-check"]));
            platform.fs().create_dir_all("src").unwrap();
            platform.fs().write("src/main.rs", b"fn main() {}").unwrap();

            let result = sources(&platform);
            assert_eq!(result, vec![PathBuf::from("src/main.rs")]);
        }

        #[test]
        fn sources_as_cargo_subcommand_with_arg_should_return_specific_source() {
            let platform = test_platform().with_env(TestEnv::new().with_args(vec![
                "program",
                "test-check",
                "src/main.rs",
            ]));
            platform.fs().create_dir_all("src/check").unwrap();
            platform.fs().write("src/main.rs", b"fn main() {}").unwrap();
            platform.fs().write("src/check/check.rs", b"").unwrap();

            let result = sources(&platform);
            assert_eq!(result, vec![PathBuf::from("src/main.rs")]);
        }

        #[test]
        fn sources_with_dir_arg_should_return_specific_source() {
            let platform =
                test_platform().with_env(TestEnv::new().with_args(vec!["program", "src/check"]));
            platform.fs().create_dir_all("src/check").unwrap();
            platform.fs().write("src/main.rs", b"fn main() {}").unwrap();
            platform.fs().write("src/check/check.rs", b"").unwrap();

            let result = sources(&platform);
            assert_eq!(result, vec![PathBuf::from("src/check/check.rs")]);
        }

        #[test]
        fn sources_with_arg_should_ignore_dir_with_rs_suffix() {
            let platform =
                test_platform().with_env(TestEnv::new().with_args(vec!["program", "src/check.rs"]));
            platform.fs().create_dir_all("src/check.rs").unwrap();
            platform.fs().write("src/main.rs", b"fn main() {}").unwrap();
            let result = sources(&platform);
            assert_eq!(result, Vec::<PathBuf>::new());
        }

        #[test]
        fn sources_with_nonexistent_arg_should_return_no_sources() {
            let platform = test_platform()
                .with_env(TestEnv::new().with_args(vec!["program", "src/nonexistent"]));
            let result = sources(&platform);
            assert_eq!(result, Vec::<PathBuf>::new());
        }

        #[test]
        fn sources_should_return_no_sources_when_none_exist() {
            let platform = test_platform();
            platform.fs().create_dir_all("src/check").unwrap();
            platform
                .fs()
                .write("src/main.cpp", b"fn main() {}")
                .unwrap();
            platform.fs().write("src/check/check.cpp", b"").unwrap();
            let result = sources(&platform);
            assert_eq!(result, Vec::<PathBuf>::new());
        }

        #[test]
        fn sources_should_ignore_dir_with_rs_suffix() {
            let platform = test_platform();
            platform.fs().create_dir_all("src/check.rs").unwrap();
            platform.fs().write("src/main.rs", b"fn main() {}").unwrap();
            let result = sources(&platform);
            assert_eq!(result, vec![PathBuf::from("src/main.rs")]);
        }

        #[test]
        fn sources_should_include_integration_tests_of_crates() {
            let platform = test_platform();

            for dir in [
                "tests",
                "agplatform/tests",
                "agplatform/src",
                "src/parser/tests",
                "fixtures/tests",
            ] {
                platform.fs().create_dir_all(dir).unwrap();
            }

            for file in [
                "Cargo.toml",
                "tests/root_test.rs",
                "agplatform/Cargo.toml",
                "agplatform/tests/platform_test.rs",
                "agplatform/src/lib.rs",
                "src/parser/tests/mod.rs",
                "fixtures/tests/x.rs",
            ] {
                platform.fs().write(file, b"").unwrap();
            }

            let mut result = sources(&platform);
            result.sort();
            assert_eq!(
                result,
                vec![
                    PathBuf::from("agplatform/src/lib.rs"),
                    PathBuf::from("agplatform/tests/platform_test.rs"),
                    PathBuf::from("fixtures/tests/x.rs"),
                    PathBuf::from("src/parser/tests/mod.rs"),
                    PathBuf::from("tests/root_test.rs"),
                ]
            );
        }

        #[test]
        fn sources_should_skip_tagged_cache_dirs() {
            let platform = test_platform();

            for dir in ["target/debug/build/out", "build_cache", "src"] {
                platform.fs().create_dir_all(dir).unwrap();
            }

            for (file, content) in [
                (
                    "target/CACHEDIR.TAG",
                    format!("{CACHEDIR_SIGNATURE}\n# cargo"),
                ),
                ("target/debug/build/out/match.gen.rs", String::new()),
                ("build_cache/CACHEDIR.TAG", String::from("Signature: other")),
                ("build_cache/lib.rs", String::new()),
                ("src/lib.rs", String::new()),
            ] {
                platform.fs().write(file, content.as_bytes()).unwrap();
            }

            let mut result = sources(&platform);
            result.sort();
            assert_eq!(
                result,
                vec![
                    PathBuf::from("build_cache/lib.rs"),
                    PathBuf::from("src/lib.rs"),
                ]
            );
        }

        #[test]
        fn sources_no_args_should_ignore_missing_current_dir() {
            let platform = test_platform().with_env(TestEnv::new().with_current_dir("/tmp"));
            let result = sources(&platform);
            assert_eq!(result, Vec::<PathBuf>::new());
        }
    }

    mod read_sources_in {
        use super::*;

        #[test]
        fn read_sources_in_should_skip_unreadable_entries() {
            let platform = test_platform();
            platform.fs().create_dir_all("src").unwrap();
            platform.fs().write("src/lib.rs", b"").unwrap();

            let result = read_sources_in(
                &mut std::iter::once(Err(agplatform::Error::fs("unreadable")))
                    .chain(platform.fs().read_dir("src").unwrap()),
                &platform,
            );

            assert_eq!(
                result,
                vec![platform.env().current_dir().join("src").join("lib.rs")]
            );
        }
    }
}
