use std::panic::Location;
use std::path::Path;
use std::path::PathBuf;

/// Enabled by the `testing` feature flag.
///
/// A real directory for tests and benchmarks, created empty and removed
/// with its content when dropped.
///
/// The name is derived from the source location of the caller, e.g.
/// `agplatform_src_fs.rs.42.21.testdir`, so that it is unique per call
/// site and any leftover points back to the code that created it. A
/// call site executed repeatedly (in a loop or through a helper function
/// without `#[track_caller]`) reuses the same directory.
///
/// When dropped during a panic (e.g. a failing assertion) the directory
/// is kept for inspection. It is removed by the next run of the same
/// code.
///
/// # Examples
///
/// ```rust
/// use agplatform::TestDir;
///
/// let dir = TestDir::new();
/// std::fs::write(dir.path().join("file.txt"), "content").unwrap();
/// let path = dir.path().to_path_buf();
/// drop(dir);
///
/// assert!(!path.exists());
/// ```
pub struct TestDir {
    path: PathBuf,
}

impl TestDir {
    /// Creates an empty directory in the current directory. Any existing
    /// directory of the same name is removed first.
    ///
    /// # Panics
    ///
    /// Panics if the current directory cannot be determined or the
    /// directory cannot be created.
    #[track_caller]
    pub fn new() -> Self {
        Self::create(caller_path(&current_dir(), "testdir"))
    }

    /// Creates an empty directory in [`std::env::temp_dir()`] (on Unix
    /// overridable with `TMPDIR`, e.g. `TMPDIR=/dev/shm`). Any existing
    /// directory of the same name is removed first.
    ///
    /// # Panics
    ///
    /// Panics if the directory cannot be created.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::TestDir;
    ///
    /// let dir = TestDir::temp();
    ///
    /// assert!(dir.path().starts_with(std::env::temp_dir()));
    /// ```
    #[track_caller]
    pub fn temp() -> Self {
        Self::create(caller_path(&std::env::temp_dir(), "testdir"))
    }

    /// Returns the path of the directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn create(path: PathBuf) -> Self {
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap_or_else(|e| {
            panic!(
                "Test directory could not be created: {}: {e}",
                path.display()
            )
        });

        Self { path }
    }
}

impl Default for TestDir {
    /// Same as [`Self::new`].
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

/// Enabled by the `testing` feature flag.
///
/// A path to a real file for tests and benchmarks. The file is not
/// created, only removed if it exists, both on creation and when
/// dropped.
///
/// It is named and kept on panic the same way as [`TestDir`], with the
/// `testfile` extension. Prefer [`TestDir`] when a test needs more than
/// one file.
///
/// # Examples
///
/// ```rust
/// use agplatform::TestFile;
///
/// let file = TestFile::new();
/// std::fs::write(file.path(), "content").unwrap();
/// let path = file.path().to_path_buf();
/// drop(file);
///
/// assert!(!path.exists());
/// ```
pub struct TestFile {
    path: PathBuf,
}

impl TestFile {
    /// Creates a path to a file in the current directory, removing the
    /// file if it exists.
    ///
    /// # Panics
    ///
    /// Panics if the current directory cannot be determined.
    #[track_caller]
    pub fn new() -> Self {
        Self::create(caller_path(&current_dir(), "testfile"))
    }

    /// Creates a path to a file in [`std::env::temp_dir()`] (on Unix
    /// overridable with `TMPDIR`, e.g. `TMPDIR=/dev/shm`), removing the
    /// file if it exists.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::TestFile;
    ///
    /// let file = TestFile::temp();
    ///
    /// assert!(file.path().starts_with(std::env::temp_dir()));
    /// ```
    #[track_caller]
    pub fn temp() -> Self {
        Self::create(caller_path(&std::env::temp_dir(), "testfile"))
    }

    /// Returns the path of the file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn create(path: PathBuf) -> Self {
        let _ = std::fs::remove_file(&path);
        Self { path }
    }
}

impl Default for TestFile {
    /// Same as [`Self::new`].
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestFile {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

fn current_dir() -> PathBuf {
    std::env::current_dir().expect("current directory should be accessible")
}

/// Returns `parent` joined with a name unique to the caller's source
/// location: its file path with separators replaced by `_`, line, column
/// and `extension`.
#[track_caller]
fn caller_path(parent: &Path, extension: &str) -> PathBuf {
    let caller = Location::caller();

    parent.join(format!(
        "{}.{}.{}.{extension}",
        caller.file().replace(['/', '\\', ':'], "_"),
        caller.line(),
        caller.column()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs `f` and unwinds without invoking the panic hook while holding
    /// the guard it returns, returning the path it returns.
    fn unwind_holding<T>(f: impl FnOnce() -> (T, PathBuf)) -> PathBuf {
        let mut path = PathBuf::new();
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let (_guard, guarded_path) = f();
            path = guarded_path;
            std::panic::resume_unwind(Box::new(()));
        }));
        path
    }

    /// Returns the sanitized source file part of a name created by
    /// [`caller_path`].
    fn caller_file(path: &Path) -> &str {
        let name = path.file_name().unwrap().to_str().unwrap();
        name.split_once(".rs.").unwrap().0
    }

    mod test_dir {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn new_should_create_empty_dir_named_after_caller_in_current_dir() {
                let dir = TestDir::new();

                assert_eq!(dir.path().parent().unwrap(), current_dir());
                assert_eq!(caller_file(dir.path()), "agplatform_src_testing");
                assert!(dir.path().is_dir());
                assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
            }

            #[test]
            fn new_should_replace_existing_dir() {
                // GIVEN a leftover directory with content
                fn create() -> TestDir {
                    TestDir::new()
                }
                let path = unwind_holding(|| {
                    let dir = create();
                    std::fs::write(dir.path().join("leftover.txt"), "").unwrap();
                    let path = dir.path().to_path_buf();
                    (dir, path)
                });

                // WHEN created again at the same call site
                let dir = create();

                // THEN the directory is empty
                assert_eq!(dir.path(), path);
                assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
            }

            #[test]
            fn new_at_different_call_sites_should_create_different_dirs() {
                let a = TestDir::new();
                let b = TestDir::new();

                assert_ne!(a.path(), b.path());
            }
        }

        mod temp {
            use super::*;

            #[test]
            fn temp_should_create_dir_named_after_caller_in_temp_dir() {
                let dir = TestDir::temp();

                assert_eq!(dir.path().parent().unwrap(), std::env::temp_dir());
                assert_eq!(caller_file(dir.path()), "agplatform_src_testing");
                assert!(dir.path().is_dir());
            }
        }

        mod create {
            use super::*;

            #[test]
            fn create_under_file_should_panic() {
                // GIVEN a parent that is a file
                let file = TestFile::new();
                std::fs::write(file.path(), "").unwrap();

                // WHEN creating a directory in it
                let result = std::panic::catch_unwind(|| TestDir::create(file.path().join("dir")));

                // THEN it panics
                assert!(result.is_err());
            }
        }

        mod default {
            use super::*;

            #[test]
            fn default_should_create_dir_named_after_caller() {
                let dir = TestDir::default();

                assert_eq!(caller_file(dir.path()), "agplatform_src_testing");
                assert!(dir.path().is_dir());
            }
        }

        mod drop {
            use super::*;

            #[test]
            fn drop_should_remove_dir_with_content() {
                let dir = TestDir::new();
                std::fs::create_dir(dir.path().join("sub")).unwrap();
                std::fs::write(dir.path().join("sub").join("a.txt"), "").unwrap();
                let path = dir.path().to_path_buf();

                drop(dir);

                assert!(!path.exists());
            }

            #[test]
            fn drop_while_panicking_should_keep_dir() {
                let path = unwind_holding(|| {
                    let dir = TestDir::new();
                    let path = dir.path().to_path_buf();
                    (dir, path)
                });

                assert!(path.is_dir());
                std::fs::remove_dir_all(path).unwrap();
            }
        }
    }

    mod test_file {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn new_should_name_file_after_caller_in_current_dir_without_creating_it() {
                let file = TestFile::new();

                assert_eq!(file.path().parent().unwrap(), current_dir());
                assert_eq!(caller_file(file.path()), "agplatform_src_testing");
                assert_eq!(file.path().extension().unwrap(), "testfile");
                assert!(!file.path().exists());
            }

            #[test]
            fn new_should_remove_existing_file() {
                // GIVEN a leftover file
                fn create() -> TestFile {
                    TestFile::new()
                }
                let path = unwind_holding(|| {
                    let file = create();
                    std::fs::write(file.path(), "leftover").unwrap();
                    let path = file.path().to_path_buf();
                    (file, path)
                });

                // WHEN created again at the same call site
                let file = create();

                // THEN the file does not exist
                assert_eq!(file.path(), path);
                assert!(!file.path().exists());
            }
        }

        mod temp {
            use super::*;

            #[test]
            fn temp_should_name_file_after_caller_in_temp_dir_without_creating_it() {
                let file = TestFile::temp();

                assert_eq!(file.path().parent().unwrap(), std::env::temp_dir());
                assert_eq!(caller_file(file.path()), "agplatform_src_testing");
                assert!(!file.path().exists());
            }
        }

        mod default {
            use super::*;

            #[test]
            fn default_should_name_file_after_caller() {
                let file = TestFile::default();

                assert_eq!(caller_file(file.path()), "agplatform_src_testing");
            }
        }

        mod drop {
            use super::*;

            #[test]
            fn drop_should_remove_file() {
                let file = TestFile::new();
                std::fs::write(file.path(), "").unwrap();
                let path = file.path().to_path_buf();

                drop(file);

                assert!(!path.exists());
            }

            #[test]
            fn drop_while_panicking_should_keep_file() {
                let path = unwind_holding(|| {
                    let file = TestFile::new();
                    std::fs::write(file.path(), "").unwrap();
                    let path = file.path().to_path_buf();
                    (file, path)
                });

                assert!(path.is_file());
                std::fs::remove_file(path).unwrap();
            }
        }
    }
}
