use std::collections::BTreeSet;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use agplatform::Env;
use agplatform::Fs;
use agplatform::Platform;

/// What `cargo test-check [<dir|file.rs>]` checks: the Rust files under the
/// directory (the current one by default) or the file. Paths are relative to
/// the current directory when they are under it.
#[derive(Debug, Default)]
pub(crate) struct Sources {
    pub(crate) files: BTreeSet<PathBuf>,
    /// The directories of the nearest `Cargo.toml` of the files.
    pub(crate) packages: BTreeSet<PathBuf>,
    /// The directories that could not be read.
    pub(crate) errors: Vec<agsource::Error>,
}

pub(crate) fn sources(p: &impl Platform) -> Sources {
    let mut args = p.env().args().skip(1).peekable();
    args.next_if(|arg| *arg == "test-check");
    let path = Path::new(args.next().map_or(OsStr::new("."), OsString::as_os_str));
    let (files, errors) = if is_rust_source(path) {
        let readable = p.fs().open_read(path).is_ok();
        (
            readable.then(|| path.to_path_buf()).into_iter().collect(),
            Vec::new(),
        )
    } else {
        agsource::rust_files(p, path)
    };
    let current_dir = p.env().current_dir();
    let mut packages = HashMap::new();
    let mut sources = Sources {
        errors,
        ..Sources::default()
    };

    for file in files {
        let file = absolute(&current_dir, &file);
        let dir = file.parent().unwrap_or(Path::new("")).to_path_buf();
        let package = packages.entry(dir).or_insert_with_key(|dir: &PathBuf| {
            dir.ancestors()
                .find(|dir| p.fs().exists(dir.join("Cargo.toml")).unwrap_or(false))
                .map(Path::to_path_buf)
        });

        if let Some(package) = package {
            sources.packages.insert(relative(&current_dir, package));
        }

        sources.files.insert(relative(&current_dir, &file));
    }

    sources
}

/// The path from `current_dir` without `..` components (joining drops the
/// `.` ones).
fn absolute(current_dir: &Path, path: &Path) -> PathBuf {
    let mut absolute = PathBuf::new();

    for component in current_dir.join(path).components() {
        if component == Component::ParentDir {
            absolute.pop();
        } else {
            absolute.push(component);
        }
    }

    absolute
}

pub(crate) fn relative(current_dir: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(current_dir)
        .map_or_else(|_| path.to_path_buf(), Path::to_path_buf)
}

pub(crate) fn is_rust_source<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().extension().and_then(|s| s.to_str()) == Some("rs")
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::TestEnv;
    use agplatform::test_platform::ROOT;
    use agplatform::test_platform::test_platform;

    fn paths(paths: &[&str]) -> BTreeSet<PathBuf> {
        paths.iter().map(PathBuf::from).collect()
    }

    mod sources {
        use super::*;

        #[test]
        fn sources_no_args_should_return_all_sources_and_their_packages() {
            let platform = test_platform();
            platform.fs().create_dir_all("a/src/check").unwrap();
            platform.fs().create_dir_all("loose").unwrap();
            platform.fs().write("Cargo.toml", "[workspace]").unwrap();
            platform.fs().write("a/Cargo.toml", "").unwrap();
            platform
                .fs()
                .write("a/src/main.rs", "fn main() {}")
                .unwrap();
            platform.fs().write("a/src/check/check.rs", "").unwrap();
            platform.fs().write("loose/x.rs", "").unwrap();
            platform.fs().write("a/src/notes.txt", "").unwrap();

            let result = sources(&platform);

            assert_eq!(
                result.files,
                paths(&["a/src/check/check.rs", "a/src/main.rs", "loose/x.rs"])
            );
            assert_eq!(result.packages, paths(&["", "a"]));
            assert!(result.errors.is_empty(), "{:?}", result.errors);
        }

        #[test]
        fn sources_with_arg_should_return_specific_source() {
            let platform =
                test_platform().with_env(TestEnv::new().with_args(vec!["program", "src/main.rs"]));
            platform.fs().create_dir_all("src/check").unwrap();
            platform.fs().write("Cargo.toml", "").unwrap();
            platform.fs().write("src/main.rs", "fn main() {}").unwrap();
            platform.fs().write("src/check/check.rs", "").unwrap();

            let result = sources(&platform);

            assert_eq!(result.files, paths(&["src/main.rs"]));
            assert_eq!(result.packages, paths(&[""]));
        }

        #[test]
        fn sources_as_cargo_subcommand_should_return_all_sources() {
            let platform =
                test_platform().with_env(TestEnv::new().with_args(vec!["program", "test-check"]));
            platform.fs().create_dir_all("src").unwrap();
            platform.fs().write("src/main.rs", "fn main() {}").unwrap();

            let result = sources(&platform);

            assert_eq!(result.files, paths(&["src/main.rs"]));
            assert_eq!(result.packages, paths(&[]));
        }

        #[test]
        fn sources_as_cargo_subcommand_with_arg_should_return_specific_source() {
            let platform = test_platform().with_env(TestEnv::new().with_args(vec![
                "program",
                "test-check",
                "src/main.rs",
            ]));
            platform.fs().create_dir_all("src/check").unwrap();
            platform.fs().write("src/main.rs", "fn main() {}").unwrap();
            platform.fs().write("src/check/check.rs", "").unwrap();

            let result = sources(&platform);

            assert_eq!(result.files, paths(&["src/main.rs"]));
        }

        #[test]
        fn sources_with_dir_arg_should_return_its_sources_and_the_package_above() {
            let platform = test_platform().with_env(
                TestEnv::new()
                    .with_args(vec!["program", "./src/check/../check"])
                    .with_current_dir(Path::new(ROOT).join("a")),
            );
            platform.fs().create_dir_all("/a/src/check").unwrap();
            platform.fs().write("/Cargo.toml", "").unwrap();
            platform
                .fs()
                .write("/a/src/main.rs", "fn main() {}")
                .unwrap();
            platform.fs().write("/a/src/check/check.rs", "").unwrap();

            let result = sources(&platform);

            assert_eq!(result.files, paths(&["src/check/check.rs"]));
            assert_eq!(result.packages, BTreeSet::from([PathBuf::from(ROOT)]));
        }

        #[test]
        fn sources_with_arg_should_ignore_dir_with_rs_suffix() {
            let platform =
                test_platform().with_env(TestEnv::new().with_args(vec!["program", "src/check.rs"]));
            platform.fs().create_dir_all("src/check.rs").unwrap();
            platform.fs().write("src/main.rs", "fn main() {}").unwrap();

            let result = sources(&platform);

            assert_eq!(result.files, paths(&[]));
            assert_eq!(result.packages, paths(&[]));
        }

        #[test]
        fn sources_with_nonexistent_arg_should_report_it() {
            let platform = test_platform()
                .with_env(TestEnv::new().with_args(vec!["program", "src/nonexistent"]));

            let result = sources(&platform);

            assert_eq!(result.files, paths(&[]));
            assert!(
                matches!(&result.errors[..], [agsource::Error::Read { path, .. }] if path == Path::new("src/nonexistent")),
                "{:?}",
                result.errors
            );
        }

        #[test]
        fn sources_should_return_no_sources_when_none_exist() {
            let platform = test_platform();
            platform.fs().create_dir_all("src/check").unwrap();
            platform.fs().write("src/main.cpp", "fn main() {}").unwrap();
            platform.fs().write("src/check/check.cpp", "").unwrap();

            let result = sources(&platform);

            assert_eq!(result.files, paths(&[]));
            assert_eq!(result.packages, paths(&[]));
        }

        #[test]
        fn sources_should_ignore_dir_with_rs_suffix() {
            let platform = test_platform();
            platform.fs().create_dir_all("src/check.rs").unwrap();
            platform.fs().write("src/main.rs", "fn main() {}").unwrap();

            let result = sources(&platform);

            assert_eq!(result.files, paths(&["src/main.rs"]));
        }
    }

    mod relative {
        use super::*;

        #[test]
        fn relative_should_keep_paths_outside_current_dir() {
            let current_dir = Path::new(ROOT).join("a");

            assert_eq!(
                relative(&current_dir, &current_dir.join("src/lib.rs")),
                PathBuf::from("src/lib.rs")
            );
            assert_eq!(
                relative(&current_dir, &Path::new(ROOT).join("b")),
                Path::new(ROOT).join("b")
            );
        }
    }
}
