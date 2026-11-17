use crate::Error;
use agplatform::Fs;
use agplatform::Platform;
use std::path::Path;
use std::path::PathBuf;
use toml::Table;

/// The crates of the extern prelude of the crate at `root`, as cargo passes
/// them with `--extern`, from the nearest `Cargo.toml` above it:
/// - the dependencies, dev and target-specific ones included;
/// - the package's own library, unless `root` is that library;
/// - `proc_macro` for the library of a proc-macro package.
///
/// None without a manifest.
pub(crate) fn externs(p: &impl Platform, root: &Path, errors: &mut Vec<Error>) -> Vec<String> {
    let Some(path) = root
        .ancestors()
        .skip(1)
        .map(|dir| dir.join("Cargo.toml"))
        .find(|path| p.fs().exists(path).unwrap_or(false))
    else {
        return Vec::new();
    };
    let manifest = match read(p, &path) {
        Ok(manifest) => manifest,
        Err(error) => {
            errors.push(error);
            return Vec::new();
        }
    };
    let mut externs = Vec::new();
    let targets = manifest.get("target").and_then(toml::Value::as_table);
    let tables = std::iter::once(&manifest).chain(
        targets
            .into_iter()
            .flat_map(|targets| targets.values().filter_map(toml::Value::as_table)),
    );

    for table in tables {
        for kind in ["dependencies", "dev-dependencies"] {
            if let Some(dependencies) = table.get(kind).and_then(toml::Value::as_table) {
                externs.extend(dependencies.keys().map(|name| crate_name(name)));
            }
        }
    }

    let lib = manifest.get("lib").and_then(toml::Value::as_table);
    let lib_path = lib
        .and_then(|lib| lib.get("path"))
        .and_then(toml::Value::as_str)
        .unwrap_or("src/lib.rs");
    let lib_path = crate::source_crate::normalize(&path.with_file_name(lib_path));

    if lib_path == root {
        if lib
            .and_then(|lib| lib.get("proc-macro"))
            .and_then(toml::Value::as_bool)
            .unwrap_or(false)
        {
            externs.push("proc_macro".to_string());
        }
    } else if p.fs().exists(&lib_path).unwrap_or(false)
        && let Some(name) = lib
            .and_then(|lib| lib.get("name"))
            .or_else(|| {
                manifest
                    .get("package")
                    .and_then(|package| package.get("name"))
            })
            .and_then(toml::Value::as_str)
    {
        externs.push(crate_name(name));
    }

    externs
}

fn read(p: &impl Platform, path: &Path) -> Result<Table, Error> {
    let text = p.fs().read_to_string(path).map_err(|error| Error::Read {
        path: path.to_path_buf(),
        error,
    })?;

    text.parse::<Table>().map_err(|e| {
        let before = &text[..e.span().map_or(0, |span| span.start)];
        Error::Parse {
            path: PathBuf::from(path),
            line: before.matches('\n').count() + 1,
            column: before
                .rsplit('\n')
                .next()
                .unwrap_or_default()
                .chars()
                .count()
                + 1,
            message: e.message().to_string(),
        }
    })
}

fn crate_name(name: &str) -> String {
    name.replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::io::ErrorKind;

    fn platform(files: &[(&str, &str)]) -> TestPlatform {
        let platform = test_platform();

        for (path, content) in files {
            platform
                .fs()
                .create_dir_all(Path::new(path).parent().unwrap())
                .unwrap();
            platform.fs().write(path, content).unwrap();
        }

        platform
    }

    fn externs_of(platform: &TestPlatform, root: &str) -> (Vec<String>, Vec<String>) {
        let mut errors = Vec::new();
        let externs = externs(platform, Path::new(root), &mut errors);
        // WHY: paths display with `\` on Windows.
        let errors = errors
            .iter()
            .map(|e| e.to_string().replace('\\', "/"))
            .collect();
        (externs, errors)
    }

    mod externs {
        use super::*;

        #[test]
        fn externs_should_list_the_dependencies_of_the_nearest_manifest() {
            let platform = platform(&[
                ("/c/Cargo.toml", "[dependencies]\nouter = \"1\""),
                (
                    "/c/m/Cargo.toml",
                    "[package]
name = \"m\"

[dependencies]
serde-json = \"1\"

[dev-dependencies]
dev = \"1\"

[build-dependencies]
build = \"1\"

[target.'cfg(unix)'.dependencies]
unix-only = \"1\"

[target.'cfg(windows)'.dev-dependencies]
windows-dev = \"1\"",
                ),
                ("/c/m/src/lib.rs", ""),
            ]);

            let externs = externs_of(&platform, "/c/m/src/lib.rs");

            assert_eq!(
                externs,
                (
                    vec![
                        "serde_json".to_string(),
                        "dev".to_string(),
                        "unix_only".to_string(),
                        "windows_dev".to_string(),
                    ],
                    vec![]
                )
            );
        }

        #[test]
        fn externs_other_targets_should_include_the_library() {
            let platform = platform(&[
                ("/c/Cargo.toml", "[package]\nname = \"my-crate\""),
                ("/c/src/lib.rs", ""),
                (
                    "/d/Cargo.toml",
                    "[package]\nname = \"d\"\n[lib]\nname = \"custom\"\npath = \"lib/x.rs\"",
                ),
                ("/d/lib/x.rs", ""),
                ("/e/Cargo.toml", "[workspace]"),
                ("/e/src/lib.rs", ""),
            ]);

            let tests = externs_of(&platform, "/c/tests/t.rs");
            let custom = externs_of(&platform, "/d/src/main.rs");
            let lib = externs_of(&platform, "/d/lib/x.rs");
            let unnamed = externs_of(&platform, "/e/tests/t.rs");

            assert_eq!(tests, (vec!["my_crate".to_string()], vec![]));
            assert_eq!(custom, (vec!["custom".to_string()], vec![]));
            assert_eq!(lib, (vec![], vec![]));
            assert_eq!(unnamed, (vec![], vec![]));
        }

        #[test]
        fn externs_without_a_library_should_not_include_the_package() {
            let platform = platform(&[("/c/Cargo.toml", "[package]\nname = \"bin\"")]);

            let externs = externs_of(&platform, "/c/src/main.rs");

            assert_eq!(externs, (vec![], vec![]));
        }

        #[test]
        fn externs_proc_macro_library_should_include_proc_macro() {
            let platform = platform(&[
                (
                    "/c/Cargo.toml",
                    "[package]\nname = \"derive\"\n[lib]\nproc-macro = true",
                ),
                ("/c/src/lib.rs", ""),
            ]);

            let lib = externs_of(&platform, "/c/src/lib.rs");
            let tests = externs_of(&platform, "/c/tests/t.rs");

            assert_eq!(lib, (vec!["proc_macro".to_string()], vec![]));
            assert_eq!(tests, (vec!["derive".to_string()], vec![]));
        }

        #[test]
        fn externs_without_a_manifest_should_be_empty() {
            let platform = platform(&[("/c/src/lib.rs", "")]);

            let externs = externs_of(&platform, "/c/src/lib.rs");

            assert_eq!(externs, (vec![], vec![]));
        }

        #[test]
        fn externs_invalid_manifest_should_be_reported() {
            let platform = platform(&[
                ("/c/Cargo.toml", "[package]\nname = \"c\"\nversion = 1.2.3"),
                ("/d/Cargo.toml", ""),
            ]);
            platform
                .fs
                .set_io_error(FsOp::Open, "/d/Cargo.toml", ErrorKind::PermissionDenied);

            let invalid = externs_of(&platform, "/c/src/lib.rs");
            let unreadable = externs_of(&platform, "/d/src/lib.rs");

            assert_eq!(
                invalid,
                (
                    vec![],
                    vec!["/c/Cargo.toml:3:14: invalid float, expected nothing".to_string()]
                )
            );
            assert_eq!(unreadable.0, Vec::<String>::new());
            assert!(unreadable.1[0].starts_with("/d/Cargo.toml: cannot read: "));
        }
    }
}
