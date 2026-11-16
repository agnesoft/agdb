use crate::Error;
use agplatform::DirEntry;
use agplatform::FileType;
use agplatform::Fs;
use agplatform::Platform;
use std::path::Path;
use std::path::PathBuf;

const CACHEDIR_SIGNATURE: &str = "Signature: 8a477f597d28d172789f06886806bc55";

/// All `.rs` files under `dir`, sorted, skipping cache directories such as
/// cargo's `target`. Directories that cannot be read are skipped and
/// returned as errors.
pub fn rust_files(p: &impl Platform, dir: impl AsRef<Path>) -> (Vec<PathBuf>, Vec<Error>) {
    let mut files = Vec::new();
    let mut errors = Vec::new();
    walk(p, dir.as_ref(), &mut files, &mut errors);
    files.sort();
    (files, errors)
}

fn walk(p: &impl Platform, dir: &Path, files: &mut Vec<PathBuf>, errors: &mut Vec<Error>) {
    let entries = match p.fs().read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            errors.push(Error::Read {
                path: dir.to_path_buf(),
                error,
            });
            return;
        }
    };

    for entry in entries {
        match entry {
            Ok(entry) => visit(p, &entry, files, errors),
            Err(error) => errors.push(Error::Read {
                path: dir.to_path_buf(),
                error,
            }),
        }
    }
}

fn visit(p: &impl Platform, entry: &DirEntry, files: &mut Vec<PathBuf>, errors: &mut Vec<Error>) {
    match entry.file_type() {
        FileType::File if entry.path().extension().is_some_and(|e| e == "rs") => {
            files.push(entry.path().to_path_buf());
        }
        FileType::Dir if !is_cache_dir(p, entry.path()) => walk(p, entry.path(), files, errors),
        _ => {}
    }
}

/// Cargo tags its target directories, whatever their name, with a
/// `CACHEDIR.TAG` (https://bford.info/cachedir/).
fn is_cache_dir(p: &impl Platform, dir: &Path) -> bool {
    p.fs()
        .read_to_string(dir.join("CACHEDIR.TAG"))
        .is_ok_and(|tag| tag.starts_with(CACHEDIR_SIGNATURE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::io::ErrorKind;

    mod rust_files {
        use super::*;

        #[test]
        fn rust_files_should_return_nested_sources_sorted() {
            let platform = test_platform();
            platform.fs().create_dir_all("/c/src/a").unwrap();
            platform.fs().write("/c/src/lib.rs", "").unwrap();
            platform.fs().write("/c/src/a/b.rs", "").unwrap();
            platform.fs().write("/c/src/a/notes.txt", "").unwrap();

            let (files, errors) = rust_files(&platform, "/c");

            assert_eq!(
                files,
                vec![
                    PathBuf::from("/c/src/a/b.rs"),
                    PathBuf::from("/c/src/lib.rs")
                ]
            );
            assert!(errors.is_empty(), "{errors:?}");
        }

        #[test]
        fn rust_files_should_skip_cache_dirs() {
            let platform = test_platform();
            platform.fs().create_dir_all("/c/out/debug").unwrap();
            platform.fs().create_dir_all("/c/tagged").unwrap();
            platform
                .fs()
                .write("/c/out/CACHEDIR.TAG", CACHEDIR_SIGNATURE)
                .unwrap();
            platform.fs().write("/c/out/debug/build.rs", "").unwrap();
            platform
                .fs()
                .write("/c/tagged/CACHEDIR.TAG", "other")
                .unwrap();
            platform.fs().write("/c/tagged/kept.rs", "").unwrap();

            let (files, _) = rust_files(&platform, "/c");

            assert_eq!(files, vec![PathBuf::from("/c/tagged/kept.rs")]);
        }

        #[test]
        fn rust_files_unreadable_dir_should_report_it() {
            let platform = test_platform();
            platform.fs().create_dir_all("/c/locked").unwrap();
            platform.fs().write("/c/lib.rs", "").unwrap();
            platform.fs().write("/c/locked/a.rs", "").unwrap();
            platform
                .fs
                .set_io_error(FsOp::ReadDir, "/c/locked", ErrorKind::PermissionDenied);

            let (files, errors) = rust_files(&platform, "/c");

            assert_eq!(files, vec![PathBuf::from("/c/lib.rs")]);
            assert!(
                matches!(&errors[..], [Error::Read { path, .. }] if path == Path::new("/c/locked")),
                "{errors:?}"
            );
        }

        #[test]
        fn rust_files_unreadable_entry_should_report_it() {
            let platform = test_platform();
            platform.fs().create_dir_all("/c").unwrap();
            platform.fs().write("/c/lib.rs", "").unwrap();
            platform.fs().write("/c/bad.rs", "").unwrap();
            platform
                .fs
                .set_io_error(FsOp::ReadDirEntry, "/c/bad.rs", ErrorKind::PermissionDenied);

            let (files, errors) = rust_files(&platform, "/c");

            assert_eq!(files, vec![PathBuf::from("/c/lib.rs")]);
            assert!(
                matches!(&errors[..], [Error::Read { path, .. }] if path == Path::new("/c")),
                "{errors:?}"
            );
        }
    }
}
