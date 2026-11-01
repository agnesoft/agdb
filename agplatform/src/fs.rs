use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::File;
use std::io::IoSlice;
use std::io::IoSliceMut;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

use crate::Error;
use crate::Result;
use crate::env::EnvImpl;

macro_rules! fs_error {
    ($kind:ident; $($arg:tt)*) => {
        $crate::Error::fs(format!($($arg)*))
            .with_source(std::io::Error::from(std::io::ErrorKind::$kind))
    };
    ($kind:expr => $($arg:tt)*) => {
        $crate::Error::fs(format!($($arg)*)).with_source(std::io::Error::from($kind))
    };
    ($($arg:tt)*) => {
        $crate::Error::fs(format!($($arg)*))
    };
}

/// A filesystem interface following [`std::fs`].
///
/// Relative paths are resolved against the current directory of the
/// platform [`Env`](crate::Env) by joining them onto it, the rest (`..`,
/// symlinks) is resolved by the OS as with [`std::fs`]. An empty path does
/// not exist. Symlinks are followed unless documented otherwise. Failures
/// carry the [`std::io::Error`] as their
/// [`source`](std::error::Error::source) and name the path they occurred
/// on. On Windows a path ending with a separator, under a file, or naming a
/// directory where a file is expected fails with the kind Unix reports.
///
/// The two provided implementations are `FsImpl`, a thin wrapper of
/// [`std::fs`], and [`TestFs`](crate::test_platform::test_fs::TestFs), an
/// in-memory double behind the `testing` feature. `TestFs` is stricter than
/// the OS: it allows one writer or many readers per file and lets tests
/// inject failures, so code passing with it does not fail on these in
/// production.
///
/// # Examples
///
/// ```rust
/// use agplatform::{Fs, Platform};
///
/// let platform = agplatform::platform();
/// let fs = platform.fs();
/// # let fs = agplatform::test_platform::test_fs::TestFs::new();
/// fs.write("/hello.txt", "world").unwrap();
/// let content = fs.read_to_string("/hello.txt").unwrap();
/// assert_eq!(content, "world");
/// ```
pub trait Fs: Send + Sync {
    /// A read-only file handle, [`Send`] and [`Sync`] so that it can be
    /// read positionally from several threads.
    type ReadHandle: ReadFile + Send + Sync;

    /// A read-write file handle, [`Send`] and [`Sync`] so that it can be
    /// read and written positionally from several threads.
    type WriteHandle: WriteFile + Send + Sync;

    /// An iterator over the entries of a directory.
    type Dir: Iterator<Item = Result<DirEntry>> + Send;

    /// Opens a file for reading, like [`std::fs::File::open`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file does not exist.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    /// use std::io::Read;
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.bin", &[0xCA, 0xFE]).unwrap();
    ///
    /// let mut handle = fs.open_read("/data.bin").unwrap();
    /// let mut buf = [0u8; 2];
    /// handle.read_exact(&mut buf).unwrap();
    /// assert_eq!(buf, [0xCA, 0xFE]);
    /// ```
    fn open_read<P: AsRef<Path>>(&self, path: P) -> Result<Self::ReadHandle>;

    /// Opens a file for reading and writing, creating it if it does not
    /// exist, see [`open_write_options`](Fs::open_write_options) with
    /// `OpenWriteOptions::new().create()`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The parent directory does not exist.
    /// - The path is a directory.
    /// - Permission denied, e.g. the file cannot be read. Use
    ///   [`write`](Fs::write) for a file that cannot be read.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    /// use std::io::{Read, Write, Seek, SeekFrom};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// let mut handle = fs.open_write("/log.txt").unwrap();
    /// handle.write_all(b"hello").unwrap();
    /// handle.seek(SeekFrom::Start(0)).unwrap();
    /// let mut buf = String::new();
    /// handle.read_to_string(&mut buf).unwrap();
    /// assert_eq!(buf, "hello");
    /// ```
    fn open_write<P: AsRef<Path>>(&self, path: P) -> Result<Self::WriteHandle> {
        self.open_write_options(path, OpenWriteOptions::new().create())
    }

    /// Opens a file for writing, and reading unless
    /// [`write_only`](OpenWriteOptions::write_only), with the given
    /// `options`, like [`std::fs::OpenOptions::open`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `create_new` is set and anything, including a dangling symlink,
    ///   already exists at the path.
    /// - Neither `create` nor `create_new` is set and the file does not
    ///   exist.
    /// - `truncate` is combined with `append`.
    /// - The path is a directory.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, OpenWriteOptions, Platform};
    /// use std::io::Write;
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "old content").unwrap();
    ///
    /// let mut handle = fs.open_write_options(
    ///     "/data.txt",
    ///     OpenWriteOptions::new().truncate(),
    /// ).unwrap();
    /// handle.write_all(b"new").unwrap();
    /// drop(handle);
    ///
    /// assert_eq!(fs.read_to_string("/data.txt").unwrap(), "new");
    /// ```
    fn open_write_options<P: AsRef<Path>>(
        &self,
        path: P,
        options: OpenWriteOptions,
    ) -> Result<Self::WriteHandle>;

    /// Reads the entire contents of a file, like [`std::fs::read`].
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or read, e.g. it does
    /// not exist or is a directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.bin", &[1, 2, 3]).unwrap();
    /// let bytes = fs.read("/data.bin").unwrap();
    /// assert_eq!(bytes, [1, 2, 3]);
    /// ```
    fn read<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        self.open_read(path).and_then(|mut file| {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map(|_| buffer)
                .map_err(fs_error_at(file.path()))
        })
    }

    /// Reads the entire contents of a file as a UTF-8 string, like
    /// [`std::fs::read_to_string`].
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, see
    /// [`read`](Fs::read), or its contents are not valid UTF-8, with the
    /// [`std::io::ErrorKind::InvalidData`] kind.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/greeting.txt", "hello").unwrap();
    /// let text = fs.read_to_string("/greeting.txt").unwrap();
    /// assert_eq!(text, "hello");
    /// ```
    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        self.open_read(path).and_then(|mut file| {
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)
                .map(|_| buffer)
                .map_err(fs_error_at(file.path()))
        })
    }

    /// Writes `data` to a file, creating it if it does not exist and
    /// truncating it if it does, like [`std::fs::write`]. The file is
    /// opened for writing only.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or written, e.g. its
    /// parent directory does not exist or it is a directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/note.txt", "hello").unwrap();
    /// assert_eq!(fs.read_to_string("/note.txt").unwrap(), "hello");
    /// ```
    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, data: C) -> Result {
        self.open_write_options(
            path,
            OpenWriteOptions::new().create().truncate().write_only(),
        )
        .and_then(|mut file| {
            file.write_all(data.as_ref())
                .map_err(fs_error_at(file.path()))
        })
    }

    /// Returns an iterator over the entries of a directory, like
    /// [`std::fs::read_dir`].
    ///
    /// The directory is read lazily as the iterator advances, so each entry
    /// is a [`Result`]. Symlinks in the directory are not followed. The `.`
    /// and `..` entries are not included. The order of entries is not
    /// guaranteed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist.
    /// - The path is not a directory.
    /// - Permission denied.
    ///
    /// The iterator yields an error if an entry cannot be read.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// # fs.create_dir_all("/mydir").unwrap();
    /// # fs.write("/mydir/a.txt", "").unwrap();
    /// let entries: Vec<_> = fs.read_dir("/mydir")
    ///     .unwrap()
    ///     .collect::<agplatform::Result<_>>()
    ///     .unwrap();
    /// assert_eq!(entries[0].file_name(), "a.txt");
    /// ```
    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Self::Dir>;

    /// Creates a directory, like [`std::fs::create_dir`].
    ///
    /// The creation is atomic: if the directory is created by several
    /// callers at once, exactly one of them succeeds. This makes it usable
    /// e.g. as a lock between processes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Anything, including a dangling symlink, already exists at `path`.
    /// - The parent directory of `path` does not exist.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.create_dir("/lock").unwrap();
    /// assert!(fs.create_dir("/lock").is_err());
    /// ```
    fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result;

    /// Creates a directory and all of its missing parent directories, like
    /// [`std::fs::create_dir_all`]. Succeeds if the directory already
    /// exists.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Something other than a directory exists at the path or at one of
    ///   its parents.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.create_dir_all("/a/b/c").unwrap();
    /// assert!(fs.read_dir("/a/b/c").unwrap().next().is_none());
    /// ```
    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result;

    /// Returns `Ok(true)` if the path points at an existing entry, like
    /// [`std::fs::exists`]. Symlinks are followed: a dangling symlink does
    /// not exist.
    ///
    /// This does not prevent time-of-check to time-of-use (TOCTOU) races.
    ///
    /// # Errors
    ///
    /// Returns an error if the existence cannot be determined, e.g. a
    /// parent component is a file or permission is denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// # fs.create_dir_all("/my/dir").unwrap();
    /// # fs.write("/my/dir/file.txt", "").unwrap();
    /// assert!(fs.exists("/my/dir/").unwrap());
    /// assert!(fs.exists("/my/dir/file.txt").unwrap());
    /// ```
    fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool>;

    /// Returns the [`Metadata`] of an entry, like [`std::fs::metadata`].
    /// Symlinks are followed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// let metadata = fs.metadata("/data.txt").unwrap();
    /// assert!(metadata.file_type.is_file());
    /// assert_eq!(metadata.len, 5);
    /// ```
    fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<Metadata>;

    /// Returns the [`Metadata`] of an entry without following a symlink at
    /// `path`, like [`std::fs::symlink_metadata`]: its own metadata is
    /// returned with [`FileType::Symlink`].
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// fs.create_symlink_file("/link", "/data.txt").unwrap();
    /// assert!(fs.symlink_metadata("/link").unwrap().file_type.is_symlink());
    /// ```
    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> Result<Metadata>;

    /// Creates a symlink at `path` pointing to the file `target`, which
    /// need not exist. A relative `target` is resolved against the parent
    /// directory of `path` when the symlink is followed.
    ///
    /// On Unix a symlink has no kind, so this is the same as
    /// [`create_symlink_dir`](Fs::create_symlink_dir). On Windows a file
    /// symlink is created, like `std::os::windows::fs::symlink_file`, which
    /// cannot be used as a directory. The `target` is stored with `/`
    /// replaced by `\` there, which Windows requires to resolve a relative
    /// symlink. Creating symlinks requires Developer Mode or administrator
    /// privileges on Windows.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Anything, including a dangling symlink, already exists at `path`.
    /// - The parent directory of `path` does not exist.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.create_dir_all("/dir").unwrap();
    /// fs.write("/dir/data.txt", "hello").unwrap();
    /// fs.create_symlink_file("/dir/link", "data.txt").unwrap();
    /// assert_eq!(fs.read_to_string("/dir/link").unwrap(), "hello");
    /// ```
    fn create_symlink_file<P: AsRef<Path>, T: AsRef<Path>>(&self, path: P, target: T) -> Result;

    /// Creates a symlink at `path` pointing to the directory `target`, which
    /// need not exist, like [`create_symlink_file`](Fs::create_symlink_file)
    /// does for a file. On Windows a directory symlink is created, like
    /// `std::os::windows::fs::symlink_dir`, which is removed by
    /// [`remove_dir`](Fs::remove_dir) and [`remove_file`](Fs::remove_file).
    ///
    /// # Errors
    ///
    /// Same as [`create_symlink_file`](Fs::create_symlink_file).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.create_symlink_dir("/link", "/dir").unwrap();
    /// fs.create_dir("/dir").unwrap();
    /// assert!(fs.metadata("/link").unwrap().file_type.is_dir());
    /// ```
    fn create_symlink_dir<P: AsRef<Path>, T: AsRef<Path>>(&self, path: P, target: T) -> Result;

    /// Returns the target of a symlink as stored, like
    /// [`std::fs::read_link`]: on Unix as given to
    /// [`create_symlink_file`](Fs::create_symlink_file). On Windows `/` is
    /// returned as `\` and an absolute target in its normalized form, e.g.
    /// `C:/x` as `C:\x`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist.
    /// - The path is not a symlink.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    /// use std::path::Path;
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.create_symlink_file("/link", "data.txt").unwrap();
    /// assert_eq!(fs.read_link("/link").unwrap(), Path::new("data.txt"));
    /// ```
    fn read_link<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf>;

    /// Returns the absolute path of `path` with all symlinks and `..`
    /// resolved, like [`std::fs::canonicalize`]. On Windows `FsImpl` returns
    /// a verbatim (`\\?\`) path like std does.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path or a symlink in it does not resolve to an existing entry.
    /// - A component of the path is not a directory.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.create_dir_all("/dir").unwrap();
    /// fs.write("/dir/data.txt", "hello").unwrap();
    /// fs.create_symlink_file("/link", "/dir").unwrap();
    /// let path = fs.canonicalize("/link/data.txt").unwrap();
    /// assert_eq!(fs.read_to_string(path).unwrap(), "hello");
    /// ```
    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf>;

    /// Creates a hard link at `path` to `target`: another name for the same
    /// file, like [`std::fs::hard_link`] with the arguments swapped to match
    /// [`create_symlink_file`](Fs::create_symlink_file). Unlike in a
    /// symlink a relative `target` is resolved like any other path. A
    /// symlink `target` is not followed where the platform allows it: the
    /// hard link is to the symlink itself.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Anything, including a dangling symlink, already exists at `path`.
    /// - The parent directory of `path` does not exist.
    /// - The `target` does not exist or is a directory.
    /// - The `target` is on a different filesystem than `path`.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// fs.create_hard_link("/other.txt", "/data.txt").unwrap();
    /// fs.write("/other.txt", "world").unwrap();
    /// assert_eq!(fs.read_to_string("/data.txt").unwrap(), "world");
    /// ```
    fn create_hard_link<P: AsRef<Path>, T: AsRef<Path>>(&self, path: P, target: T) -> Result;

    /// Copies the content and the permissions of the file `source` to
    /// `target`, overwriting it if it exists, like [`std::fs::copy`].
    /// Returns the number of bytes copied. Symlinks are followed, at both
    /// `source` and `target`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `source` does not exist or is not a file.
    /// - The parent directory of `target` does not exist.
    /// - The `target` is a directory.
    /// - Permission denied, e.g. the `target` is read-only, or on Windows
    ///   hidden while the `source` is not.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// assert_eq!(fs.copy("/data.txt", "/copy.txt").unwrap(), 5);
    /// fs.write("/copy.txt", "world").unwrap();
    /// assert_eq!(fs.read_to_string("/data.txt").unwrap(), "hello");
    /// ```
    fn copy<S: AsRef<Path>, T: AsRef<Path>>(&self, source: S, target: T) -> Result<u64>;

    /// Copies the directory `source` with all of its content to the new
    /// path `target`. The `source` itself is followed if it is a symlink,
    /// the symlinks in it are copied as symlinks with the same target and
    /// kind, see [`create_symlink_dir`](Fs::create_symlink_dir), files are
    /// copied as by [`copy`](Fs::copy). Directories get the permissions of their source
    /// once their content is copied, so read-only directories are copied
    /// too.
    ///
    /// The whole tree is listed before anything is copied, so `target` may
    /// be inside `source`. If an error occurs, what was copied until then is
    /// not removed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `source` does not exist or is not a directory.
    /// - Anything, including a dangling symlink, already exists at `target`.
    /// - The parent directory of `target` does not exist.
    /// - An entry cannot be read or copied, e.g. it is neither a file, a
    ///   directory nor a symlink.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.create_dir_all("/dir/sub").unwrap();
    /// fs.write("/dir/sub/data.txt", "hello").unwrap();
    /// fs.copy_dir_all("/dir", "/copy").unwrap();
    /// assert_eq!(fs.read_to_string("/copy/sub/data.txt").unwrap(), "hello");
    /// ```
    fn copy_dir_all<S: AsRef<Path>, T: AsRef<Path>>(&self, source: S, target: T) -> Result;

    /// Removes a file or a symlink, like [`std::fs::remove_file`]. A symlink
    /// is not followed: the symlink itself is removed. A file with other
    /// hard links remains under its other names. On Windows a directory
    /// symlink is removed too, which [`std::fs::remove_file`] refuses.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist.
    /// - The path is a directory.
    /// - Permission denied, e.g. on Windows the file is open or read-only.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// fs.remove_file("/data.txt").unwrap();
    /// assert!(!fs.exists("/data.txt").unwrap());
    /// ```
    fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result;

    /// Removes an empty directory, like [`std::fs::remove_dir`]. On Windows
    /// a directory symlink is removed as well, on Unix a symlink is not a
    /// directory.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist.
    /// - The path is not a directory or the directory is not empty.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.create_dir("/dir").unwrap();
    /// fs.remove_dir("/dir").unwrap();
    /// assert!(!fs.exists("/dir").unwrap());
    /// ```
    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result;

    /// Removes a directory with all of its content, like
    /// [`std::fs::remove_dir_all`]. Symlinks are not followed: a symlink at
    /// `path` is removed itself, as are the symlinks in the directory, their
    /// targets are kept. If an error occurs, what was removed until then is
    /// not restored.
    ///
    /// Unlike std, a `path` that resolves to a root, e.g. `/` or `/home/..`,
    /// is refused, as is a symlink followed by a trailing separator, e.g.
    /// `link/`, whose target's content std would remove before failing.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist.
    /// - The path is a file, a root or a symlink with a trailing separator.
    /// - An entry cannot be removed, e.g. permission denied or on Windows a
    ///   file is open.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.create_dir_all("/dir/sub").unwrap();
    /// fs.write("/dir/sub/data.txt", "hello").unwrap();
    /// fs.remove_dir_all("/dir").unwrap();
    /// assert!(!fs.exists("/dir").unwrap());
    /// ```
    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result;

    /// Renames (moves) a file, a directory or a symlink `source` to
    /// `target`, replacing `target` atomically, like [`std::fs::rename`]. A
    /// symlink is not followed: the symlink itself is renamed. A directory
    /// `source` replaces only an empty directory, anything else replaces
    /// anything but a directory. On Unix renaming a hard link onto another
    /// one of the same file does nothing.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `source` does not exist.
    /// - The `source` is a directory and `target` is inside it or is not an
    ///   empty directory.
    /// - The `source` is not a directory and `target` is a directory.
    /// - The parent directory of `target` does not exist.
    /// - The `source` and `target` are on different filesystems.
    /// - Permission denied, e.g. on Windows a file is open.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt.tmp", "hello").unwrap();
    /// fs.rename("/data.txt.tmp", "/data.txt").unwrap();
    /// assert_eq!(fs.read_to_string("/data.txt").unwrap(), "hello");
    /// ```
    fn rename<S: AsRef<Path>, T: AsRef<Path>>(&self, source: S, target: T) -> Result;

    /// Sets or clears the [`readonly`](Metadata::readonly) attribute of an
    /// entry. Symlinks are followed. On Unix clearing it gives write access
    /// back to the owner only, unlike
    /// [`std::fs::Permissions::set_readonly`] which makes the entry
    /// world-writable.
    ///
    /// Does nothing if the attribute already has the requested value, so the
    /// call succeeds even if the caller would not be allowed to change it.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist.
    /// - Permission denied, e.g. the caller does not own the entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// fs.set_readonly("/data.txt", true).unwrap();
    /// assert!(fs.metadata("/data.txt").unwrap().readonly);
    /// ```
    fn set_readonly<P: AsRef<Path>>(&self, path: P, readonly: bool) -> Result;

    /// Sets or clears the [`executable`](Metadata::executable) attribute of
    /// a file. Symlinks are followed. Setting it adds the execute bits for
    /// the owner and those who can read the file unless one is set already,
    /// clearing it removes all of them.
    ///
    /// Unix only: on Windows executability is derived from the file
    /// extension and cannot be set.
    ///
    /// Does nothing if the attribute already has the requested value, so the
    /// call succeeds even if the caller would not be allowed to change it.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist.
    /// - The path is a directory, whose execute bits grant traversal.
    /// - Permission denied, e.g. the caller does not own the file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/run.sh", "echo hello").unwrap();
    /// fs.set_executable("/run.sh", true).unwrap();
    /// assert!(fs.metadata("/run.sh").unwrap().executable);
    /// ```
    #[cfg(unix)]
    fn set_executable<P: AsRef<Path>>(&self, path: P, executable: bool) -> Result;

    /// Sets the [`modified`](Metadata::modified) time of an entry, like
    /// [`std::fs::set_times`]. Symlinks are followed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist.
    /// - Permission denied.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    /// use std::time::SystemTime;
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// fs.set_modified("/data.txt", SystemTime::UNIX_EPOCH).unwrap();
    /// assert_eq!(fs.metadata("/data.txt").unwrap().modified, SystemTime::UNIX_EPOCH);
    /// ```
    fn set_modified<P: AsRef<Path>>(&self, path: P, modified: SystemTime) -> Result;

    /// Sets the [`mode`](Metadata::mode) of an entry, like
    /// [`std::fs::set_permissions`]: the permission bits including the
    /// set-user-ID, set-group-ID and sticky bits. Bits above `0o7777` are
    /// ignored. Symlinks are followed.
    ///
    /// Unix only: Windows has no mode, std supports only the
    /// [`readonly`](Metadata::readonly) attribute there.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist.
    /// - Permission denied, e.g. the caller does not own the entry.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/secret.txt", "hello").unwrap();
    /// fs.set_mode("/secret.txt", 0o600).unwrap();
    /// assert_eq!(fs.metadata("/secret.txt").unwrap().mode, 0o600);
    /// ```
    #[cfg(unix)]
    fn set_mode<P: AsRef<Path>>(&self, path: P, mode: u32) -> Result;
}

/// An open file that can be read, see [`Fs::open_read`].
pub trait ReadFile: Read + Seek {
    /// Returns the path the file was opened at, resolved against the
    /// current directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, ReadFile};
    /// use std::path::Path;
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// # use agplatform::test_platform::ROOT;
    /// fs.write("/data.txt", "hello").unwrap();
    /// let file = fs.open_read("/data.txt").unwrap();
    /// assert_eq!(file.path(), Path::new(ROOT).join("data.txt"));
    /// ```
    fn path(&self) -> &Path;

    /// Returns the metadata of the open file, like
    /// [`std::fs::File::metadata`].
    ///
    /// # Errors
    ///
    /// Returns an error if the metadata cannot be queried.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, ReadFile};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// let file = fs.open_read("/data.txt").unwrap();
    /// assert_eq!(file.metadata().unwrap().len, 5);
    /// ```
    fn metadata(&self) -> Result<Metadata>;

    /// Reads into `buf` from the `offset` of the file, like
    /// `std::os::unix::fs::FileExt::read_at` and
    /// `std::os::windows::fs::FileExt::seek_read`. Returns the number of
    /// bytes read, `0` at or beyond the end of the file. Short reads are
    /// possible, see [`read_exact_at`](Self::read_exact_at).
    ///
    /// Takes `&self`, so that one handle can be read at different offsets
    /// from several threads. The position of the handle used by
    /// [`Read`] and [`Seek`] is unspecified afterwards: it is kept on Unix
    /// and moved past the bytes read on Windows and in `TestFs`.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, ReadFile};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// let file = fs.open_read("/data.txt").unwrap();
    /// let mut buf = [0u8; 3];
    /// assert_eq!(file.read_at(&mut buf, 2).unwrap(), 3);
    /// assert_eq!(&buf, b"llo");
    /// ```
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize>;

    /// Reads exactly `buf.len()` bytes from the `offset` of the file by
    /// repeated [`read_at`](Self::read_at) calls, like
    /// `std::os::unix::fs::FileExt::read_exact_at`. Calls interrupted by a
    /// signal are retried.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, caused by an I/O error
    /// of the [`std::io::ErrorKind::UnexpectedEof`] kind if it ends before
    /// `buf` is filled. The content of `buf` is unspecified then.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, ReadFile};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// let file = fs.open_read("/data.txt").unwrap();
    /// let mut buf = [0u8; 4];
    /// file.read_exact_at(&mut buf, 1).unwrap();
    /// assert_eq!(&buf, b"ello");
    /// assert!(file.read_exact_at(&mut buf, 2).is_err());
    /// ```
    fn read_exact_at(&self, mut buf: &mut [u8], mut offset: u64) -> Result {
        while !buf.is_empty() {
            match self.read_at(buf, offset) {
                Ok(0) => {
                    return Err(fs_error!(
                        UnexpectedEof;
                        "File could not be read, it ends before the buffer is filled: {}",
                        self.path().display()
                    ));
                }
                Ok(read) => {
                    buf = &mut buf[read..];
                    offset += read as u64;
                }
                Err(error) if is_interrupted(&error) => {}
                Err(error) => return Err(error),
            }
        }

        Ok(())
    }

    /// Locks the file for reading, waiting until no other handle holds an
    /// exclusive lock on it, like [`std::fs::File::lock_shared`]. Any
    /// number of handles may hold a shared lock at once. The lock is
    /// released by [`unlock`](Self::unlock) or when the handle is dropped.
    /// On Unix the lock is advisory: it does not stop reads and writes of
    /// those not locking the file.
    ///
    /// # Errors
    ///
    /// Returns an error if locking is not supported, e.g. on some network
    /// filesystems.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, ReadFile};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// let file = fs.open_read("/data.txt").unwrap();
    /// file.lock_shared().unwrap();
    /// ```
    fn lock_shared(&self) -> Result;

    /// Locks the file for reading like [`lock_shared`](Self::lock_shared)
    /// if possible right away, like [`std::fs::File::try_lock_shared`].
    /// Returns `false` if another handle holds an exclusive lock on it.
    ///
    /// # Errors
    ///
    /// Returns an error if locking is not supported, e.g. on some network
    /// filesystems.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, ReadFile};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// let file = fs.open_read("/data.txt").unwrap();
    /// assert!(file.try_lock_shared().unwrap());
    /// ```
    fn try_lock_shared(&self) -> Result<bool>;

    /// Releases the lock held by this handle, if any, like
    /// [`std::fs::File::unlock`].
    ///
    /// # Errors
    ///
    /// Returns an error if the lock cannot be released.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, ReadFile};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// let file = fs.open_read("/data.txt").unwrap();
    /// file.lock_shared().unwrap();
    /// file.unlock().unwrap();
    /// ```
    fn unlock(&self) -> Result;
}

/// An open file that can be written, see [`Fs::open_write`].
pub trait WriteFile: ReadFile + Write {
    /// Writes `buf` at the `offset` of the file, like
    /// `std::os::unix::fs::FileExt::write_at` and
    /// `std::os::windows::fs::FileExt::seek_write`, extending the file if
    /// needed. Returns the number of bytes written. Short writes are
    /// possible, see [`write_all_at`](Self::write_all_at).
    ///
    /// Takes `&self` like [`read_at`](ReadFile::read_at), after which the
    /// position of the handle is unspecified as well. On Linux the `offset`
    /// is ignored if the file is opened for
    /// [`append`](OpenWriteOptions::append).
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written, e.g. the disk is
    /// full.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, WriteFile};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// let file = fs.open_write("/data.txt").unwrap();
    /// assert_eq!(file.write_at(b"J", 0).unwrap(), 1);
    /// drop(file);
    /// assert_eq!(fs.read_to_string("/data.txt").unwrap(), "Jello");
    /// ```
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize>;

    /// Writes all of `buf` at the `offset` of the file by repeated
    /// [`write_at`](Self::write_at) calls, like
    /// `std::os::unix::fs::FileExt::write_all_at`. Calls interrupted by a
    /// signal are retried.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written, caused by an I/O
    /// error of the [`std::io::ErrorKind::WriteZero`] kind if a call writes
    /// nothing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, WriteFile};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// let file = fs.open_write("/data.txt").unwrap();
    /// file.write_all_at(b"world", 6).unwrap();
    /// file.write_all_at(b"hello ", 0).unwrap();
    /// drop(file);
    /// assert_eq!(fs.read_to_string("/data.txt").unwrap(), "hello world");
    /// ```
    fn write_all_at(&self, mut buf: &[u8], mut offset: u64) -> Result {
        while !buf.is_empty() {
            match self.write_at(buf, offset) {
                Ok(0) => {
                    return Err(fs_error!(
                        WriteZero;
                        "File could not be written, no bytes were written: {}",
                        self.path().display()
                    ));
                }
                Ok(written) => {
                    buf = &buf[written..];
                    offset += written as u64;
                }
                Err(error) if is_interrupted(&error) => {}
                Err(error) => return Err(error),
            }
        }

        Ok(())
    }

    /// Writes all of `bufs` at the position of the handle, at the end of the
    /// file if it is opened for [`append`](OpenWriteOptions::append), by
    /// repeated [`write_vectored`](Write::write_vectored) calls, like the
    /// unstable `Write::write_all_vectored`. On Unix a call writes all of
    /// them at once if it can, e.g. a record of several parts appended to a
    /// log by one system call. On Windows each call writes only the first
    /// non-empty buffer. Calls interrupted by a signal are retried. `bufs`
    /// is left in an unspecified state.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written, caused by an I/O
    /// error of the [`std::io::ErrorKind::WriteZero`] kind if a call writes
    /// nothing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, OpenWriteOptions, Platform, WriteFile};
    /// use std::io::IoSlice;
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/log", "a").unwrap();
    /// let mut file = fs
    ///     .open_write_options("/log", OpenWriteOptions::new().append())
    ///     .unwrap();
    /// file.write_all_slices(&mut [IoSlice::new(b"b"), IoSlice::new(b"cd")]).unwrap();
    /// drop(file);
    /// assert_eq!(fs.read_to_string("/log").unwrap(), "abcd");
    /// ```
    fn write_all_slices(&mut self, mut bufs: &mut [IoSlice<'_>]) -> Result {
        IoSlice::advance_slices(&mut bufs, 0);

        while !bufs.is_empty() {
            match self.write_vectored(bufs) {
                Ok(0) => {
                    return Err(fs_error!(
                        WriteZero;
                        "File could not be written, no bytes were written: {}",
                        self.path().display()
                    ));
                }
                Ok(written) => IoSlice::advance_slices(&mut bufs, written),
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                Err(error) => return Err(fs_error_at(self.path())(error)),
            }
        }

        Ok(())
    }

    /// Truncates or extends the open file to `len` bytes, like
    /// [`std::fs::File::set_len`]. An extension is filled with zeros. The
    /// position of the handle is not moved.
    ///
    /// # Errors
    ///
    /// Returns an error if the length cannot be set, e.g. the file is in
    /// append mode on Windows.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, ReadFile, WriteFile};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// let file = fs.open_write("/data.txt").unwrap();
    /// file.set_len(2).unwrap();
    /// assert_eq!(file.metadata().unwrap().len, 2);
    /// ```
    fn set_len(&self, len: u64) -> Result;

    /// Writes the content and the metadata of the open file to the disk,
    /// like [`std::fs::File::sync_all`].
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be written to the disk.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, WriteFile};
    /// use std::io::Write;
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// let mut file = fs.open_write("/data.txt").unwrap();
    /// file.write_all(b"hello").unwrap();
    /// file.sync_all().unwrap();
    /// ```
    fn sync_all(&self) -> Result;

    /// Writes the content of the open file to the disk, like
    /// [`std::fs::File::sync_data`]. Metadata is written only if needed to
    /// read the content back, e.g. a changed length.
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be written to the disk.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, WriteFile};
    /// use std::io::Write;
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// let mut file = fs.open_write("/data.txt").unwrap();
    /// file.write_all(b"hello").unwrap();
    /// file.sync_data().unwrap();
    /// ```
    fn sync_data(&self) -> Result;

    /// Locks the file exclusively, waiting until no other handle holds a
    /// lock on it, like [`std::fs::File::lock`]. The lock is released by
    /// [`unlock`](ReadFile::unlock) or when the handle is dropped. On Unix
    /// the lock is advisory: it does not stop reads and writes of those not
    /// locking the file. On Windows other handles cannot read or write the
    /// locked file. Locking a handle that already holds a lock is platform
    /// dependent, as with std: on Windows it fails or waits forever.
    ///
    /// # Errors
    ///
    /// Returns an error if locking is not supported, e.g. on some network
    /// filesystems, or on Windows if the file is opened for appending only.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, ReadFile, WriteFile};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// let file = fs.open_write("/data.txt").unwrap();
    /// file.lock().unwrap();
    /// file.unlock().unwrap();
    /// ```
    fn lock(&self) -> Result;

    /// Locks the file exclusively like [`lock`](Self::lock) if possible
    /// right away, like [`std::fs::File::try_lock`]. Returns `false` if
    /// another handle holds a lock on it.
    ///
    /// # Errors
    ///
    /// Returns an error if locking is not supported, e.g. on some network
    /// filesystems, or on Windows if the file is opened for appending only.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, Platform, WriteFile};
    ///
    /// let platform = agplatform::platform();
    /// let fs = platform.fs();
    /// # let fs = agplatform::test_platform::test_fs::TestFs::new();
    /// let file = fs.open_write("/data.txt").unwrap();
    /// assert!(file.try_lock().unwrap());
    /// ```
    fn try_lock(&self) -> Result<bool>;
}

/// Options for opening a file for writing, see
/// [`Fs::open_write_options`], like [`std::fs::OpenOptions`] with `write`
/// set.
///
/// # Examples
///
/// ```rust
/// use agplatform::OpenWriteOptions;
///
/// let options = OpenWriteOptions::new()
///     .create()
///     .truncate();
/// ```
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OpenWriteOptions {
    /// Writes are appended to the end of the file regardless of the
    /// position.
    pub append: bool,
    /// Truncate the file to zero length when opened.
    pub truncate: bool,
    /// Create the file if it does not exist.
    pub create: bool,
    /// Create a new file, failing if it already exists.
    pub create_new: bool,
    /// Open the file for writing only, see [`write_only`](Self::write_only).
    pub write_only: bool,
    /// The mode a new file is created with, see
    /// [`mode`](Self::mode) (Unix only).
    #[cfg(unix)]
    pub mode: Option<u32>,
}

impl OpenWriteOptions {
    /// Returns a new set of options with all flags disabled.
    pub const fn new() -> Self {
        OpenWriteOptions {
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            write_only: false,
            #[cfg(unix)]
            mode: None,
        }
    }

    /// Sets append mode: every write goes to the end of the file regardless
    /// of the position, e.g. for log files.
    pub const fn append(mut self) -> Self {
        self.append = true;
        self
    }

    /// Creates the file if it does not already exist.
    pub const fn create(mut self) -> Self {
        self.create = true;
        self
    }

    /// Creates a new file, failing if anything already exists at the path,
    /// atomically: of several callers creating it at once exactly one
    /// succeeds.
    pub const fn create_new(mut self) -> Self {
        self.create_new = true;
        self
    }

    /// Opens the file for writing only, like [`std::fs::write`], so that a
    /// file that cannot be read can be written. Reading from the handle
    /// fails.
    pub const fn write_only(mut self) -> Self {
        self.write_only = true;
        self
    }

    /// Truncates the file to zero length when opened. Cannot be combined
    /// with [`append`](Self::append).
    pub const fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }

    /// Sets the [`mode`](Metadata::mode) a new file is created with, as
    /// restricted by the umask of the process. Without it a file is created
    /// with `0o666` restricted by the umask. An existing file keeps its mode.
    ///
    /// Unix only: Windows has no mode, see [`Fs::set_mode`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::OpenWriteOptions;
    ///
    /// let options = OpenWriteOptions::new().create_new().mode(0o600);
    /// ```
    #[cfg(unix)]
    pub const fn mode(mut self, mode: u32) -> Self {
        self.mode = Some(mode);
        self
    }
}

/// Distinguishes files, directories and symlinks in a [`DirEntry`] or
/// [`Metadata`].
///
/// # Examples
///
/// ```rust
/// use agplatform::FileType;
///
/// let ft = FileType::Dir;
/// assert!(ft.is_dir());
/// assert!(!ft.is_file());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// A regular file.
    File,
    /// A directory.
    Dir,
    /// A symlink, reported only where symlinks are not followed:
    /// [`Fs::read_dir`] and [`Fs::symlink_metadata`].
    Symlink,
    /// Anything else, e.g. a FIFO, a socket or a device on Unix.
    Other,
}

impl FileType {
    /// Returns `true` if this entry is a directory.
    pub fn is_dir(&self) -> bool {
        matches!(self, FileType::Dir)
    }

    /// Returns `true` if this entry is a regular file.
    pub fn is_file(&self) -> bool {
        matches!(self, FileType::File)
    }

    /// Returns `true` if this entry is a symlink.
    pub fn is_symlink(&self) -> bool {
        matches!(self, FileType::Symlink)
    }
}

impl From<std::fs::FileType> for FileType {
    fn from(file_type: std::fs::FileType) -> Self {
        if file_type.is_symlink() {
            return FileType::Symlink;
        }

        if file_type.is_dir() {
            return FileType::Dir;
        }

        // WHY: std reports every other entry as a file on Windows.
        #[cfg(unix)]
        if !file_type.is_file() {
            return FileType::Other;
        }

        FileType::File
    }
}

/// An entry of a directory, returned by iterating the result of
/// [`Fs::read_dir`].
///
/// # Examples
///
/// ```rust
/// use agplatform::{Fs, Platform};
///
/// let platform = agplatform::platform();
/// let fs = platform.fs();
/// # let fs = agplatform::test_platform::test_fs::TestFs::new();
/// # fs.create_dir_all("/mydir").unwrap();
/// # fs.write("/mydir/a.txt", "").unwrap();
/// for entry in fs.read_dir("/mydir").unwrap() {
///     let entry = entry.unwrap();
///     println!("{}: {:?}", entry.path().display(), entry.file_type());
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    path: PathBuf,
    file_type: FileType,
}

impl DirEntry {
    pub(crate) fn new(path: PathBuf, file_type: FileType) -> Self {
        Self { path, file_type }
    }

    /// Returns the path of this entry: the directory path as given to
    /// [`Fs::read_dir`], resolved against the current directory, joined
    /// with the [`file_name`](Self::file_name).
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the name of this entry, which need not be valid UTF-8.
    pub fn file_name(&self) -> &OsStr {
        self.path.file_name().unwrap_or_default()
    }

    /// Returns the type of this entry. A symlink is not followed.
    pub fn file_type(&self) -> FileType {
        self.file_type
    }
}

/// [internal]
///
/// The flags of [`Metadata`]: natively stored on Windows, derived from the
/// mode on Unix (see `UnixAttributes`), or with the platform semantics
/// applied by [`platform_attributes`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Attributes {
    pub(crate) readonly: bool,
    pub(crate) executable: bool,
    pub(crate) hidden: bool,
}

impl Attributes {
    /// Returns the natively stored flags, which on Windows are the
    /// attributes themselves, see `UnixAttributes::attributes`.
    #[cfg(windows)]
    pub(crate) fn attributes(&self) -> Attributes {
        *self
    }
}

/// [internal]
///
/// The mode and owner of a Unix entry as natively stored, from which its
/// [`Attributes`] derive.
#[cfg(unix)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UnixAttributes {
    pub(crate) mode: u32,
    pub(crate) uid: u32,
    pub(crate) gid: u32,
}

#[cfg(unix)]
impl UnixAttributes {
    pub(crate) fn attributes(&self) -> Attributes {
        Attributes {
            readonly: self.mode & 0o222 == 0,
            executable: self.mode & 0o111 != 0,
            hidden: false,
        }
    }
}

/// [internal]
///
/// The attributes of an entry as natively stored by the platform, shared by
/// [`FsImpl`] and [`TestFs`](crate::test_platform::test_fs::TestFs).
#[cfg(unix)]
pub(crate) type NativeAttributes = UnixAttributes;
/// [internal]
///
/// The attributes of an entry as natively stored by the platform, shared by
/// [`FsImpl`] and [`TestFs`](crate::test_platform::test_fs::TestFs).
#[cfg(windows)]
pub(crate) type NativeAttributes = Attributes;

/// [internal]
///
/// Returns the Unix `mode` with the write bits set as
/// [`Fs::set_readonly`] does: all cleared if `readonly`, otherwise the
/// owner's added if none is set.
#[cfg(unix)]
pub(crate) fn readonly_mode(mode: u32, readonly: bool) -> u32 {
    if readonly {
        mode & !0o222
    } else if mode & 0o222 == 0 {
        mode | 0o200
    } else {
        mode
    }
}

/// [internal]
///
/// Returns the Unix `mode` with the execute bits set as
/// [`Fs::set_executable`] does: all cleared unless `executable`, otherwise
/// added for the owner and those who can read if none is set.
#[cfg(unix)]
pub(crate) fn executable_mode(mode: u32, executable: bool) -> u32 {
    if !executable {
        mode & !0o111
    } else if mode & 0o111 == 0 {
        mode | ((mode & 0o444) >> 2) | 0o100
    } else {
        mode
    }
}

/// Metadata of an entry, returned by [`Fs::metadata`].
///
/// The `readonly`, `executable` and `hidden` flags map to the native
/// concept of the current platform. Where there is none, the flag is
/// derived from the path and cannot be set on that platform. On Unix the
/// `mode` and owner are reported as well, Windows has no equivalent in std:
///
/// | Flag         | Unix                                   | Windows                                             |
/// |--------------|----------------------------------------|-----------------------------------------------------|
/// | `readonly`   | no write permission bits               | `FILE_ATTRIBUTE_READONLY`                           |
/// | `executable` | any execute permission bit, files only | derived: `.exe`, `.com`, `.bat` or `.cmd` extension |
/// | `hidden`     | derived: name starts with `.`          | `FILE_ATTRIBUTE_HIDDEN`                             |
///
/// # Examples
///
/// ```rust
/// use agplatform::{Fs, Platform};
///
/// let platform = agplatform::platform();
/// let fs = platform.fs();
/// # let fs = agplatform::test_platform::test_fs::TestFs::new();
/// fs.write("/data.txt", "hello").unwrap();
/// let metadata = fs.metadata("/data.txt").unwrap();
/// assert!(!metadata.readonly);
/// assert!(metadata.modified <= std::time::SystemTime::now());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    /// The type of the entry, a symlink only when returned by
    /// [`Fs::symlink_metadata`].
    pub file_type: FileType,
    /// The size of a file in bytes. The value for other entries is platform
    /// specific, e.g. for a symlink the length of its target on Unix and
    /// `0` on Windows.
    pub len: u64,
    /// The last modification time, or [`SystemTime::UNIX_EPOCH`] on
    /// platforms that do not record it.
    pub modified: SystemTime,
    /// The creation time, or [`SystemTime::UNIX_EPOCH`] if the platform or
    /// filesystem does not record it (e.g. Linux before 4.11 or
    /// filesystems without `statx` birth time support).
    pub created: SystemTime,
    /// The entry cannot be written to, see [`Fs::set_readonly`].
    pub readonly: bool,
    /// The file can be executed, see `Fs::set_executable` (Unix only).
    pub executable: bool,
    /// The entry is hidden from regular directory listings.
    pub hidden: bool,
    /// The entry is a directory symlink, see [`Fs::create_symlink_dir`]
    /// (Windows only).
    #[cfg(windows)]
    pub symlink_dir: bool,
    /// The permission bits including the set-user-ID, set-group-ID and
    /// sticky bits (`0o7777`), see `Fs::set_mode` (Unix only).
    #[cfg(unix)]
    pub mode: u32,
    /// The user ID of the owner (Unix only).
    #[cfg(unix)]
    pub uid: u32,
    /// The group ID of the owner (Unix only).
    #[cfg(unix)]
    pub gid: u32,
}

impl Metadata {
    pub(crate) fn new(
        path: &Path,
        file_type: FileType,
        len: u64,
        modified: SystemTime,
        created: SystemTime,
        native: NativeAttributes,
    ) -> Self {
        let attributes = platform_attributes(path, file_type, native.attributes());

        Self {
            file_type,
            len,
            modified,
            created,
            readonly: attributes.readonly,
            executable: attributes.executable,
            hidden: attributes.hidden,
            #[cfg(windows)]
            symlink_dir: false,
            #[cfg(unix)]
            mode: native.mode,
            #[cfg(unix)]
            uid: native.uid,
            #[cfg(unix)]
            gid: native.gid,
        }
    }

    /// Returns the [`Metadata`] of the entry at `path` from its std
    /// `metadata`.
    fn from_std(path: &Path, metadata: &std::fs::Metadata) -> Self {
        let result = Self::new(
            path,
            metadata.file_type().into(),
            metadata.len(),
            metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
            metadata.created().unwrap_or(SystemTime::UNIX_EPOCH),
            native_attributes(metadata),
        );

        #[cfg(windows)]
        let result = Self {
            symlink_dir: std::os::windows::fs::FileTypeExt::is_symlink_dir(&metadata.file_type()),
            ..result
        };

        result
    }
}

#[cfg(unix)]
fn native_attributes(metadata: &std::fs::Metadata) -> NativeAttributes {
    use std::os::unix::fs::MetadataExt;

    UnixAttributes {
        mode: metadata.mode() & 0o7777,
        uid: metadata.uid(),
        gid: metadata.gid(),
    }
}

#[cfg(windows)]
fn native_attributes(metadata: &std::fs::Metadata) -> NativeAttributes {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;

    Attributes {
        readonly: metadata.permissions().readonly(),
        executable: false,
        hidden: metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0,
    }
}

/// [internal]
///
/// Applies the platform semantics described in [`Metadata`] to the natively
/// `stored` flags.
fn platform_attributes(path: &Path, file_type: FileType, stored: Attributes) -> Attributes {
    #[cfg(unix)]
    let attributes = Attributes {
        readonly: stored.readonly,
        executable: file_type.is_file() && stored.executable,
        hidden: path
            .file_name()
            .is_some_and(|name| name.as_encoded_bytes().starts_with(b".")),
    };
    #[cfg(windows)]
    let attributes = Attributes {
        readonly: stored.readonly,
        executable: file_type.is_file()
            && path.extension().is_some_and(|extension| {
                ["exe", "com", "bat", "cmd"]
                    .iter()
                    .any(|executable| extension.eq_ignore_ascii_case(executable))
            }),
        hidden: stored.hidden,
    };

    attributes
}

/// A file opened by [`FsImpl`], a [`std::fs::File`] with the path it was
/// opened at.
struct FileHandle {
    file: File,
    path: PathBuf,
}

impl FileHandle {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        #[cfg(unix)]
        let read = std::os::unix::fs::FileExt::read_at(&self.file, buf, offset);
        #[cfg(windows)]
        let read = std::os::windows::fs::FileExt::seek_read(&self.file, buf, offset);

        read.map_err(fs_error_at(&self.path))
    }

    fn metadata(&self) -> Result<Metadata> {
        self.file
            .metadata()
            .map(|metadata| Metadata::from_std(&self.path, &metadata))
            .map_err(fs_error_at(&self.path))
    }

    fn lock_shared(&self) -> Result {
        self.file.lock_shared().map_err(fs_error_at(&self.path))
    }

    fn try_lock_shared(&self) -> Result<bool> {
        self.locked(self.file.try_lock_shared())
    }

    fn unlock(&self) -> Result {
        // WHY: std on Windows fails with ERROR_NOT_LOCKED if no lock is held.
        match self.file.unlock() {
            #[cfg(windows)]
            Err(error) if error.raw_os_error() == Some(158) => Ok(()),
            result => result.map_err(fs_error_at(&self.path)),
        }
    }

    /// Returns whether a `try` locking `result` acquired the lock, `false`
    /// if another handle holds it.
    fn locked(&self, result: std::result::Result<(), std::fs::TryLockError>) -> Result<bool> {
        result
            .map(|()| true)
            .or_else(|error| {
                let error = std::io::Error::from(error);
                (error.kind() == std::io::ErrorKind::WouldBlock)
                    .then_some(false)
                    .ok_or(error)
            })
            .map_err(fs_error_at(&self.path))
    }
}

/// A read-only file handle of [`FsImpl`] backed by [`std::fs::File`].
pub struct FileReadHandle(FileHandle);

impl Read for FileReadHandle {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.file.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.0.file.read_vectored(bufs)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.0.file.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.0.file.read_to_string(buf)
    }
}

impl Seek for FileReadHandle {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.0.file.seek(pos)
    }
}

impl ReadFile for FileReadHandle {
    fn path(&self) -> &Path {
        &self.0.path
    }

    fn metadata(&self) -> Result<Metadata> {
        self.0.metadata()
    }

    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        self.0.read_at(buf, offset)
    }

    fn lock_shared(&self) -> Result {
        self.0.lock_shared()
    }

    fn try_lock_shared(&self) -> Result<bool> {
        self.0.try_lock_shared()
    }

    fn unlock(&self) -> Result {
        self.0.unlock()
    }
}

/// A read-write file handle of [`FsImpl`] backed by [`std::fs::File`].
pub struct FileWriteHandle(FileHandle);

impl Read for FileWriteHandle {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.file.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.0.file.read_vectored(bufs)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.0.file.read_to_end(buf)
    }

    fn read_to_string(&mut self, buf: &mut String) -> std::io::Result<usize> {
        self.0.file.read_to_string(buf)
    }
}

impl Seek for FileWriteHandle {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.0.file.seek(pos)
    }
}

impl Write for FileWriteHandle {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.file.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> std::io::Result<usize> {
        self.0.file.write_vectored(bufs)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.file.flush()
    }
}

impl ReadFile for FileWriteHandle {
    fn path(&self) -> &Path {
        &self.0.path
    }

    fn metadata(&self) -> Result<Metadata> {
        self.0.metadata()
    }

    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        self.0.read_at(buf, offset)
    }

    fn lock_shared(&self) -> Result {
        self.0.lock_shared()
    }

    fn try_lock_shared(&self) -> Result<bool> {
        self.0.try_lock_shared()
    }

    fn unlock(&self) -> Result {
        self.0.unlock()
    }
}

impl WriteFile for FileWriteHandle {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        #[cfg(unix)]
        let written = std::os::unix::fs::FileExt::write_at(&self.0.file, buf, offset);
        #[cfg(windows)]
        let written = std::os::windows::fs::FileExt::seek_write(&self.0.file, buf, offset);

        written.map_err(fs_error_at(&self.0.path))
    }

    fn set_len(&self, len: u64) -> Result {
        self.0.file.set_len(len).map_err(fs_error_at(&self.0.path))
    }

    fn sync_all(&self) -> Result {
        self.0.file.sync_all().map_err(fs_error_at(&self.0.path))
    }

    fn sync_data(&self) -> Result {
        self.0.file.sync_data().map_err(fs_error_at(&self.0.path))
    }

    fn lock(&self) -> Result {
        self.0.file.lock().map_err(fs_error_at(&self.0.path))
    }

    fn try_lock(&self) -> Result<bool> {
        self.0.locked(self.0.file.try_lock())
    }
}

/// A lazy iterator over the entries of a directory of [`FsImpl`], backed by
/// [`std::fs::ReadDir`].
pub struct ReadDir {
    path: PathBuf,
    dir: std::fs::ReadDir,
}

impl Iterator for ReadDir {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.dir.next().map(|entry| {
            entry
                .and_then(|entry| {
                    entry
                        .file_type()
                        .map(|file_type| DirEntry::new(entry.path(), file_type.into()))
                })
                .map_err(fs_error_at(&self.path))
        })
    }
}

/// The real filesystem implementation, a thin wrapper of [`std::fs`].
///
/// Relative paths are resolved against the current directory of its
/// [`Env`](crate::Env). Normally obtained via
/// [`platform()`](crate::platform) rather than constructed directly.
pub struct FsImpl {
    env: EnvImpl,
}

impl FsImpl {
    /// [internal]
    ///
    /// Creates a new `FsImpl` resolving relative paths against
    /// `current_dir`.
    #[cfg(test)]
    pub(crate) fn new(current_dir: PathBuf) -> Self {
        Self::with_env(EnvImpl::new(Vec::new(), current_dir))
    }

    /// [internal]
    ///
    /// Creates a new `FsImpl` resolving relative paths against the
    /// current directory of `env`.
    pub(crate) fn with_env(env: EnvImpl) -> Self {
        FsImpl { env }
    }

    /// Returns `path` resolved against the current directory, see
    /// [`resolve_path`].
    fn path<'a>(&self, path: &'a Path) -> Result<Cow<'a, Path>> {
        resolve_path(&self.env.current_dir_arc(), path)
    }

    fn open(&self, path: &Path, options: &std::fs::OpenOptions) -> Result<FileHandle> {
        let path = self.path(path)?;
        #[cfg(windows)]
        check_requires_dir(&path)?;

        match options.open(&path) {
            Ok(file) => Ok(FileHandle {
                file,
                path: path.into_owned(),
            }),
            Err(error) => {
                #[cfg(windows)]
                let error = unix_error(&path, error, is_dir_kind);
                Err(fs_error_at(&path)(error))
            }
        }
    }
}

impl Fs for FsImpl {
    type ReadHandle = FileReadHandle;
    type WriteHandle = FileWriteHandle;
    type Dir = ReadDir;

    fn open_read<P: AsRef<Path>>(&self, path: P) -> Result<Self::ReadHandle> {
        self.open(path.as_ref(), File::options().read(true))
            .map(FileReadHandle)
    }

    fn open_write_options<P: AsRef<Path>>(
        &self,
        path: P,
        options: OpenWriteOptions,
    ) -> Result<Self::WriteHandle> {
        let mut open_options = File::options();
        open_options
            .read(!options.write_only)
            .write(true)
            .create(options.create)
            .create_new(options.create_new)
            .append(options.append)
            .truncate(options.truncate);

        #[cfg(unix)]
        if let Some(mode) = options.mode {
            std::os::unix::fs::OpenOptionsExt::mode(&mut open_options, mode);
        }

        #[cfg(windows)]
        if options.create || options.create_new {
            check_create_file(&self.path(path.as_ref())?)?;
        }

        self.open(path.as_ref(), &open_options).map(FileWriteHandle)
    }

    fn read<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>> {
        let path = self.path(path.as_ref())?;
        std::fs::read(&path).map_err(fs_error_at(&path))
    }

    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let path = self.path(path.as_ref())?;
        std::fs::read_to_string(&path).map_err(fs_error_at(&path))
    }

    fn write<P: AsRef<Path>, C: AsRef<[u8]>>(&self, path: P, data: C) -> Result {
        let path = self.path(path.as_ref())?;
        std::fs::write(&path, data).map_err(fs_error_at(&path))
    }

    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Self::Dir> {
        let path = self.path(path.as_ref())?;

        std::fs::read_dir(&path)
            .map(|dir| ReadDir {
                path: path.to_path_buf(),
                dir,
            })
            .map_err(|error| {
                // WHY: Windows reports a dangling file symlink as not a
                // directory.
                #[cfg(windows)]
                let error = unix_error(&path, error, |path| {
                    std::fs::exists(path)
                        .is_ok_and(|exists| !exists)
                        .then_some(std::io::ErrorKind::NotFound)
                });
                fs_error_at(&path)(error)
            })
    }

    fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result {
        let path = self.path(path.as_ref())?;
        std::fs::create_dir(&path).map_err(|error| {
            #[cfg(windows)]
            let error = unix_error(&path, error, |_| None);
            fs_error_at(&path)(error)
        })
    }

    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result {
        let path = self.path(path.as_ref())?;
        std::fs::create_dir_all(&path).map_err(|error| {
            #[cfg(windows)]
            let error = unix_error(&path, error, |_| None);
            fs_error_at(&path)(error)
        })
    }

    fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        let path = self.path(path.as_ref())?;
        std::fs::exists(&path).map_err(fs_error_at(&path))
    }

    fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<Metadata> {
        let path = self.path(path.as_ref())?;

        std::fs::metadata(&path)
            .map(|metadata| Metadata::from_std(&path, &metadata))
            .map_err(fs_error_at(&path))
    }

    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> Result<Metadata> {
        let path = self.path(path.as_ref())?;

        std::fs::symlink_metadata(&path)
            .map(|metadata| Metadata::from_std(&path, &metadata))
            .map_err(fs_error_at(&path))
    }

    fn create_symlink_file<P: AsRef<Path>, T: AsRef<Path>>(&self, path: P, target: T) -> Result {
        let path = self.path(path.as_ref())?;

        #[cfg(unix)]
        let created = std::os::unix::fs::symlink(target, &path);
        #[cfg(windows)]
        let created =
            std::os::windows::fs::symlink_file(windows_symlink_target(target.as_ref()), &path);

        created.map_err(fs_error_at(&path))
    }

    fn create_symlink_dir<P: AsRef<Path>, T: AsRef<Path>>(&self, path: P, target: T) -> Result {
        let path = self.path(path.as_ref())?;

        #[cfg(unix)]
        let created = std::os::unix::fs::symlink(target, &path);
        #[cfg(windows)]
        let created =
            std::os::windows::fs::symlink_dir(windows_symlink_target(target.as_ref()), &path);

        created.map_err(fs_error_at(&path))
    }

    fn read_link<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let path = self.path(path.as_ref())?;
        std::fs::read_link(&path).map_err(fs_error_at(&path))
    }

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let path = self.path(path.as_ref())?;
        std::fs::canonicalize(&path).map_err(fs_error_at(&path))
    }

    fn create_hard_link<P: AsRef<Path>, T: AsRef<Path>>(&self, path: P, target: T) -> Result {
        let current_dir = self.env.current_dir_arc();
        let path = resolve_path(&current_dir, path.as_ref())?;
        let target = resolve_path(&current_dir, target.as_ref())?;
        std::fs::hard_link(&target, &path).map_err(fs_error_between(&target, &path))
    }

    fn copy<S: AsRef<Path>, T: AsRef<Path>>(&self, source: S, target: T) -> Result<u64> {
        let current_dir = self.env.current_dir_arc();
        let source = resolve_path(&current_dir, source.as_ref())?;
        let target = resolve_path(&current_dir, target.as_ref())?;
        #[cfg(windows)]
        check_create_file(&target)?;

        std::fs::copy(&source, &target).map_err(|error| {
            #[cfg(windows)]
            let error = unix_error(&target, error, is_dir_kind);
            fs_error_between(&source, &target)(error)
        })
    }

    fn copy_dir_all<S: AsRef<Path>, T: AsRef<Path>>(&self, source: S, target: T) -> Result {
        let current_dir = self.env.current_dir_arc();
        copy_tree(
            self,
            &resolve_path(&current_dir, source.as_ref())?,
            &resolve_path(&current_dir, target.as_ref())?,
        )
    }

    fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result {
        let path = self.path(path.as_ref())?;
        #[cfg(windows)]
        check_requires_dir(&path)?;

        // WHY: a directory symlink is a directory entry only `remove_dir`
        // removes on Windows, so it is tried once `remove_file` fails.
        #[cfg(windows)]
        let removed = std::fs::remove_file(&path).or_else(|error| {
            use std::os::windows::fs::FileTypeExt;

            match std::fs::symlink_metadata(&path) {
                Ok(metadata) if metadata.file_type().is_symlink_dir() => std::fs::remove_dir(&path),
                Ok(metadata) if metadata.is_dir() => {
                    Err(std::io::Error::new(std::io::ErrorKind::IsADirectory, error))
                }
                _ => Err(error),
            }
        });
        #[cfg(unix)]
        let removed = std::fs::remove_file(&path);

        removed.map_err(fs_error_at(&path))
    }

    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result {
        let path = self.path(path.as_ref())?;
        std::fs::remove_dir(&path).map_err(fs_error_at(&path))
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result {
        let path = self.path(path.as_ref())?;
        check_remove_dir_all(&path)?;

        // WHY: std on Windows refuses a file symlink it removes elsewhere.
        #[cfg(windows)]
        if std::fs::symlink_metadata(&path).is_ok_and(|metadata| {
            std::os::windows::fs::FileTypeExt::is_symlink_file(&metadata.file_type())
        }) {
            return std::fs::remove_file(&path).map_err(fs_error_at(&path));
        }

        std::fs::remove_dir_all(&path).map_err(fs_error_at(&path))
    }

    fn rename<S: AsRef<Path>, T: AsRef<Path>>(&self, source: S, target: T) -> Result {
        let current_dir = self.env.current_dir_arc();
        let source = resolve_path(&current_dir, source.as_ref())?;
        let target = resolve_path(&current_dir, target.as_ref())?;
        #[cfg(windows)]
        check_rename(&source, &target)?;
        // WHY: a directory target with a trailing separator is the
        // directory itself, which Windows may not look up.
        #[cfg(windows)]
        let target = Cow::<Path>::Owned(without_separator(&target));
        std::fs::rename(&source, &target).map_err(fs_error_between(&source, &target))
    }

    fn set_readonly<P: AsRef<Path>>(&self, path: P, readonly: bool) -> Result {
        #[cfg(unix)]
        let changed = self.set_mode_with(path.as_ref(), |metadata| {
            Ok(readonly_mode(native_attributes(metadata).mode, readonly))
        });
        #[cfg(windows)]
        let changed = {
            let path = self.path(path.as_ref())?;

            std::fs::metadata(&path)
                .and_then(|metadata| {
                    let mut permissions = metadata.permissions();

                    if permissions.readonly() == readonly {
                        return Ok(());
                    }

                    permissions.set_readonly(readonly);
                    std::fs::set_permissions(&path, permissions)
                })
                .map_err(fs_error_at(&path))
        };

        changed
    }

    #[cfg(unix)]
    fn set_executable<P: AsRef<Path>>(&self, path: P, executable: bool) -> Result {
        self.set_mode_with(path.as_ref(), |metadata| {
            if metadata.is_dir() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::IsADirectory,
                    "executable attribute cannot be set on a directory",
                ));
            }

            Ok(executable_mode(
                native_attributes(metadata).mode,
                executable,
            ))
        })
    }

    fn set_modified<P: AsRef<Path>>(&self, path: P, modified: SystemTime) -> Result {
        let path = self.path(path.as_ref())?;

        std::fs::set_times(&path, std::fs::FileTimes::new().set_modified(modified))
            .map_err(fs_error_at(&path))
    }

    #[cfg(unix)]
    fn set_mode<P: AsRef<Path>>(&self, path: P, mode: u32) -> Result {
        use std::os::unix::fs::PermissionsExt;

        let path = self.path(path.as_ref())?;

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode & 0o7777))
            .map_err(fs_error_at(&path))
    }
}

#[cfg(unix)]
impl FsImpl {
    /// Sets the mode of the entry at `path` to the one `mode` returns for
    /// its metadata, unless it is the same.
    fn set_mode_with(
        &self,
        path: &Path,
        mode: impl FnOnce(&std::fs::Metadata) -> std::io::Result<u32>,
    ) -> Result {
        use std::os::unix::fs::PermissionsExt;

        let path = self.path(path)?;

        std::fs::metadata(&path)
            .and_then(|metadata| {
                let current = native_attributes(&metadata).mode;
                let mode = mode(&metadata)?;

                // WHY: changing permissions requires owning the entry, so a
                // request that is already satisfied must not fail.
                if mode == current {
                    return Ok(());
                }

                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode))
            })
            .map_err(fs_error_at(&path))
    }
}

/// Returns `target` with `/` replaced by `\`: Windows does not resolve the
/// target of a relative symlink containing `/`.
#[cfg(windows)]
pub(crate) fn windows_symlink_target(target: &Path) -> PathBuf {
    let bytes: Vec<u8> = target
        .as_os_str()
        .as_encoded_bytes()
        .iter()
        .map(|&byte| if byte == b'/' { b'\\' } else { byte })
        .collect();

    // SAFETY: replacing one ASCII byte with another keeps the bytes valid
    // as they were obtained from `as_encoded_bytes`.
    PathBuf::from(unsafe { std::ffi::OsString::from_encoded_bytes_unchecked(bytes) })
}

/// [internal]
///
/// The entries of a directory tree listed by [`list_tree`], each as its
/// `source` and `target` path. The `dirs` are in pre-order, the root first.
#[derive(Default)]
struct Tree {
    dirs: Vec<(PathBuf, PathBuf)>,
    files: Vec<(PathBuf, PathBuf)>,
    symlinks: Vec<(PathBuf, PathBuf)>,
}

/// [internal]
///
/// Implements [`Fs::copy_dir_all`] on top of the other methods of `fs`.
pub(crate) fn copy_tree<F: Fs>(fs: &F, source: &Path, target: &Path) -> Result {
    if !fs.metadata(source)?.file_type.is_dir() {
        return Err(fs_error!(
            NotADirectory;
            "Directory could not be copied, source is not a directory: {}",
            source.display()
        ));
    }

    let mut tree = Tree::default();
    tree.dirs.push((source.to_path_buf(), target.to_path_buf()));
    list_tree(fs, source, target, &mut tree)?;

    for (_, target) in &tree.dirs {
        fs.create_dir(target)?;
    }

    for (source, target) in &tree.files {
        fs.copy(source, target)?;
    }

    for (source, target) in &tree.symlinks {
        copy_symlink(fs, source, target)?;
    }

    for (source, target) in tree.dirs.iter().rev() {
        copy_permissions(fs, source, target)?;
    }

    Ok(())
}

/// [internal]
///
/// Creates the symlink `target` with the target of the symlink `source`,
/// a directory symlink if `source` is one.
fn copy_symlink<F: Fs>(fs: &F, source: &Path, target: &Path) -> Result {
    let link = fs.read_link(source)?;

    #[cfg(windows)]
    if fs.symlink_metadata(source)?.symlink_dir {
        return fs.create_symlink_dir(target, link);
    }

    fs.create_symlink_file(target, link)
}

/// [internal]
///
/// Adds the entries of the directory `source` and of its subdirectories to
/// the `tree`, each with its path under `target`.
fn list_tree<F: Fs>(fs: &F, source: &Path, target: &Path, tree: &mut Tree) -> Result {
    fs.read_dir(source)?.try_for_each(|entry| {
        entry
            .and_then(|entry| add_entry(tree, entry, target))
            .and_then(|dir| match dir {
                Some((source, target)) => list_tree(fs, &source, &target, tree),
                None => Ok(()),
            })
    })
}

/// [internal]
///
/// Adds the directory `entry` to the `tree` with its path under `target`
/// and returns both paths if it is a directory, whose entries are to be
/// added next.
fn add_entry(
    tree: &mut Tree,
    entry: DirEntry,
    target: &Path,
) -> Result<Option<(PathBuf, PathBuf)>> {
    let paths = (entry.path().to_path_buf(), target.join(entry.file_name()));

    match entry.file_type() {
        FileType::Dir => {
            tree.dirs.push(paths.clone());
            return Ok(Some(paths));
        }
        FileType::File => tree.files.push(paths),
        FileType::Symlink => tree.symlinks.push(paths),
        FileType::Other => {
            return Err(fs_error!(
                Unsupported;
                "Directory could not be copied, entry is neither a file, a directory nor a symlink: {}",
                paths.0.display()
            ));
        }
    }

    Ok(None)
}

/// [internal]
///
/// Sets the permissions of the directory `target` to those of `source`.
fn copy_permissions<F: Fs>(fs: &F, source: &Path, target: &Path) -> Result {
    let metadata = fs.metadata(source)?;

    #[cfg(unix)]
    let copied = fs.set_mode(target, metadata.mode);
    #[cfg(windows)]
    let copied = fs.set_readonly(target, metadata.readonly);

    copied
}

/// [internal]
///
/// Fails if `path` is a root, e.g. `/`, which [`Fs::remove_dir_all`]
/// refuses before anything is removed.
pub(crate) fn check_not_root(path: &Path) -> Result {
    if path.parent().is_none() {
        return Err(fs_error!(
            ResourceBusy;
            "Directory could not be removed, path is a root: {}",
            path.display()
        ));
    }

    Ok(())
}

/// Fails if [`std::fs::remove_dir_all`] would follow the last component of
/// `path`, as it does for a last `..` or a trailing separator, to a root,
/// e.g. of `/home/..`, or to a symlink, e.g. of `link/`, whose target's
/// content it would remove before failing. A plain symlink is not followed,
/// std removes just the symlink.
fn check_remove_dir_all(path: &Path) -> Result {
    let named: PathBuf = path.components().collect();
    let last_is_parent = matches!(named.components().next_back(), Some(Component::ParentDir));

    if named.as_os_str() == path.as_os_str() && !last_is_parent {
        return check_not_root(path);
    }

    if !last_is_parent
        && std::fs::symlink_metadata(&named).is_ok_and(|metadata| metadata.file_type().is_symlink())
    {
        return Err(fs_error!(
            NotADirectory;
            "Directory could not be removed, path is a symlink: {}",
            path.display()
        ));
    }

    check_not_root(&std::fs::canonicalize(path).map_err(fs_error_at(path))?)
}

/// [internal]
///
/// Fails like Unix renaming `source` to `target`, which std on Windows may
/// do instead: a directory and an existing non-directory either way round,
/// a non-directory with a trailing separator on either path, or a
/// directory into itself.
#[cfg(windows)]
fn check_rename(source: &Path, target: &Path) -> Result {
    let is_dir = |path: &Path| {
        std::fs::symlink_metadata(without_separator(path)).map(|metadata| metadata.is_dir())
    };
    let Ok(source_dir) = is_dir(source) else {
        return Ok(());
    };
    let (named_source, named_target) = (without_separator(source), without_separator(target));

    let kind = if !source_dir && (requires_dir(source) || requires_dir(target)) {
        std::io::ErrorKind::NotADirectory
    } else if source_dir && named_target != named_source && named_target.starts_with(&named_source)
    {
        std::io::ErrorKind::InvalidInput
    } else {
        match (source_dir, is_dir(target)) {
            (true, Ok(false)) => std::io::ErrorKind::NotADirectory,
            (false, Ok(true)) => std::io::ErrorKind::IsADirectory,
            _ => return Ok(()),
        }
    };

    Err(fs_error!(
        kind =>
        "Path could not be renamed: {} -> {}",
        source.display(),
        target.display()
    ))
}

/// [internal]
///
/// Returns `path` without a trailing separator, which Windows does not
/// look up for a file.
#[cfg(windows)]
fn without_separator(path: &Path) -> PathBuf {
    path.components().collect()
}

/// [internal]
///
/// Returns `true` if `path` ends with a separator other than a root, so
/// that it must be a directory.
#[cfg(windows)]
fn requires_dir(path: &Path) -> bool {
    path.as_os_str()
        .as_encoded_bytes()
        .last()
        .is_some_and(|&byte| std::path::is_separator(byte.into()))
        && path.parent().is_some()
}

/// [internal]
///
/// Fails like Unix if `path` [`requires_dir`] but is an entry other than a
/// directory, which Windows may look up as a file.
#[cfg(windows)]
fn check_requires_dir(path: &Path) -> Result {
    if requires_dir(path)
        && std::fs::metadata(without_separator(path)).is_ok_and(|metadata| !metadata.is_dir())
    {
        return Err(fs_error!(NotADirectory; "Not a directory: {}", path.display()));
    }

    Ok(())
}

/// [internal]
///
/// Fails like Unix creating a file at `path` that [`requires_dir`]: as a
/// directory if its parent is one, as its parent is missing otherwise.
#[cfg(windows)]
fn check_create_file(path: &Path) -> Result {
    if !requires_dir(path) {
        return Ok(());
    }

    let named = without_separator(path);
    let kind = match named.parent().map(std::fs::metadata) {
        Some(Ok(metadata)) if metadata.is_dir() => std::io::ErrorKind::IsADirectory,
        _ => missing_kind(&named),
    };

    Err(fs_error!(kind => "File could not be created: {}", path.display()))
}

/// [internal]
///
/// Returns the kind Unix reports for the missing `path`: not a directory
/// if its closest existing ancestor is not one, not found otherwise.
#[cfg(windows)]
fn missing_kind(path: &Path) -> std::io::ErrorKind {
    match without_separator(path)
        .ancestors()
        .skip(1)
        .find_map(|ancestor| std::fs::metadata(ancestor).ok())
    {
        Some(metadata) if !metadata.is_dir() => std::io::ErrorKind::NotADirectory,
        _ => std::io::ErrorKind::NotFound,
    }
}

/// [internal]
///
/// Returns the `error` of an operation on `path` with the kind Unix
/// reports: not a directory if an ancestor of `path` is not one, else the
/// one `kind` returns for the `path`, if any.
#[cfg(windows)]
fn unix_error(
    path: &Path,
    error: std::io::Error,
    kind: fn(&Path) -> Option<std::io::ErrorKind>,
) -> std::io::Error {
    let unix_kind = match missing_kind(path) {
        std::io::ErrorKind::NotADirectory => Some(std::io::ErrorKind::NotADirectory),
        _ => kind(path),
    };

    match unix_kind {
        Some(unix_kind) if unix_kind != error.kind() => std::io::Error::new(unix_kind, error),
        _ => error,
    }
}

/// [internal]
///
/// Returns [`IsADirectory`](std::io::ErrorKind::IsADirectory) if `path` is
/// a directory, for [`unix_error`] of an operation expecting a file.
#[cfg(windows)]
fn is_dir_kind(path: &Path) -> Option<std::io::ErrorKind> {
    std::fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_dir())
        .then_some(std::io::ErrorKind::IsADirectory)
}

/// [internal]
///
/// Returns `path` joined onto `current_dir` unless it is absolute. Fails if
/// it is empty, which does not exist as with [`std::fs`] rather than being
/// the current directory.
pub(crate) fn resolve_path<'a>(current_dir: &Path, path: &'a Path) -> Result<Cow<'a, Path>> {
    if path.as_os_str().is_empty() {
        return Err(fs_error!(NotFound; "Path not found, it is empty"));
    }

    Ok(if path.is_absolute() {
        Cow::Borrowed(path)
    } else {
        Cow::Owned(current_dir.join(path))
    })
}

/// [internal]
///
/// Returns a mapping into an [`Error::fs`] whose description is the mapped
/// error followed by the `path` it occurred on and whose source is the
/// mapped error, for use with [`Result::map_err`].
pub(crate) fn fs_error_at<T: std::error::Error + Send + Sync + 'static>(
    path: &Path,
) -> impl FnOnce(T) -> Error + '_ {
    move |error| fs_error!("{error}: {}", path.display()).with_source(error)
}

/// [internal]
///
/// Returns whether the `error` is caused by an interrupted I/O call, which
/// is to be retried.
fn is_interrupted(error: &Error) -> bool {
    error
        .source_as::<std::io::Error>()
        .is_some_and(|error| error.kind() == std::io::ErrorKind::Interrupted)
}

/// [internal]
///
/// Returns a mapping like [`fs_error_at`] for an error that occurred on
/// both the `source` and the `target` path.
pub(crate) fn fs_error_between<'a, T: std::error::Error + Send + Sync + 'static>(
    source: &'a Path,
    target: &'a Path,
) -> impl FnOnce(T) -> Error + 'a {
    move |error| {
        fs_error!("{error}: {} -> {}", source.display(), target.display()).with_source(error)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::io::SeekFrom;

    use super::*;
    use crate::TestDir;
    use crate::TestFs;
    use crate::test_platform::test_fs::FsOp;

    /// Returns a test directory of the caller and an `FsImpl` whose current
    /// directory it is.
    #[track_caller]
    fn test_fs() -> (TestDir, FsImpl) {
        let test_dir = TestDir::new();
        let fs = FsImpl::new(test_dir.path().to_path_buf());
        (test_dir, fs)
    }

    /// Returns the kind of the [`std::io::Error`] that caused the `error`.
    fn io_kind(error: &Error) -> Option<std::io::ErrorKind> {
        error
            .source_as::<std::io::Error>()
            .map(std::io::Error::kind)
    }

    #[cfg(unix)]
    fn set_mode(path: &Path, mode: u32) {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
    }

    #[cfg(windows)]
    fn hide(path: &Path) {
        let status = std::process::Command::new("attrib")
            .arg("+h")
            .arg(path)
            .status()
            .unwrap();
        assert!(status.success(), "attrib +h failed: {status}");
    }

    pub(crate) mod fs {
        use super::*;

        pub(crate) mod open_read {
            use super::*;

            pub(crate) fn open_read_should_open_relative_and_absolute_path(
                fs: &impl Fs,
                current_dir: &Path,
            ) {
                fs.write("a.txt", "content").unwrap();
                let absolute = current_dir.join("a.txt").display().to_string();

                for path in ["a.txt", absolute.as_str()] {
                    let mut buf = String::new();
                    fs.open_read(path)
                        .unwrap()
                        .read_to_string(&mut buf)
                        .unwrap();

                    assert_eq!(buf, "content", "path: {path}");
                }
            }

            pub(crate) fn open_read_missing_file_should_fail_with_path(
                fs: &impl Fs,
                current_dir: &Path,
            ) {
                fs.write("a.txt", "").unwrap();
                fs.create_symlink_file("dangling", "missing").unwrap();

                let error = fs.open_read("missing").err().unwrap();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
                assert!(
                    error
                        .description()
                        .ends_with(&format!(": {}", current_dir.join("missing").display())),
                    "{error}"
                );
                assert_eq!(
                    io_kind(&fs.open_read("dangling").err().unwrap()),
                    Some(std::io::ErrorKind::NotFound)
                );
                for path in ["a.txt/b", "a.txt/"] {
                    assert_eq!(
                        fs.open_read(path)
                            .map(|_| ())
                            .map_err(|error| io_kind(&error)),
                        Err(Some(std::io::ErrorKind::NotADirectory)),
                        "path: {path}"
                    );
                }
            }
        }

        pub(crate) mod open_write {
            use super::*;

            pub(crate) fn open_write_should_create_file_without_truncating(fs: &impl Fs) {
                fs.open_write("new.txt").unwrap();
                fs.write("a.txt", "content").unwrap();

                fs.open_write("a.txt").unwrap().write_all(b"C").unwrap();

                assert_eq!(fs.read_to_string("new.txt").unwrap(), "");
                assert_eq!(fs.read_to_string("a.txt").unwrap(), "Content");
            }

            pub(crate) fn open_write_should_open_for_reading(fs: &impl Fs) {
                let mut file = fs.open_write("a.txt").unwrap();
                file.write_all(b"abc").unwrap();
                file.seek(SeekFrom::Start(1)).unwrap();
                let mut buf = String::new();

                file.read_to_string(&mut buf).unwrap();

                assert_eq!(buf, "bc");
            }
        }

        pub(crate) mod open_write_options {
            use super::*;

            pub(crate) fn open_write_options_should_append_or_truncate(fs: &impl Fs) {
                fs.write("a.txt", "content").unwrap();

                fs.open_write_options("a.txt", OpenWriteOptions::new().append())
                    .unwrap()
                    .write_all(b"!")
                    .unwrap();
                assert_eq!(fs.read_to_string("a.txt").unwrap(), "content!");

                fs.open_write_options("a.txt", OpenWriteOptions::new().truncate())
                    .unwrap()
                    .write_all(b"new")
                    .unwrap();
                assert_eq!(fs.read_to_string("a.txt").unwrap(), "new");
            }

            pub(crate) fn open_write_options_append_should_write_at_end_regardless_of_position(
                fs: &impl Fs,
            ) {
                fs.write("a.txt", "content").unwrap();
                let mut file = fs
                    .open_write_options("a.txt", OpenWriteOptions::new().append())
                    .unwrap();

                file.seek(SeekFrom::Start(0)).unwrap();
                file.write_all(b"!").unwrap();

                assert_eq!(file.stream_position().unwrap(), 8);
                drop(file);
                assert_eq!(fs.read_to_string("a.txt").unwrap(), "content!");
            }

            pub(crate) fn open_write_options_write_only_should_not_be_readable(fs: &impl Fs) {
                fs.write("a.txt", "content").unwrap();
                let mut file = fs
                    .open_write_options("a.txt", OpenWriteOptions::new().write_only())
                    .unwrap();

                assert!(file.read_to_end(&mut Vec::new()).is_err());
                assert!(file.read_at(&mut [0; 1], 0).is_err());
            }

            pub(crate) fn open_write_options_without_create_missing_file_should_fail(fs: &impl Fs) {
                let error = fs
                    .open_write_options("a.txt", OpenWriteOptions::new())
                    .err()
                    .unwrap();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
                assert!(!fs.exists("a.txt").unwrap());
            }

            pub(crate) fn open_write_options_create_new_existing_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();
                fs.create_symlink_file("link", "missing").unwrap();

                for path in ["a.txt", "link"] {
                    let error = fs
                        .open_write_options(path, OpenWriteOptions::new().create_new())
                        .err()
                        .unwrap();

                    assert_eq!(
                        io_kind(&error),
                        Some(std::io::ErrorKind::AlreadyExists),
                        "path: {path}"
                    );
                }
                assert!(!fs.exists("missing").unwrap());
            }

            pub(crate) fn open_write_options_append_and_truncate_should_require_create_new(
                fs: &impl Fs,
            ) {
                let error = fs
                    .open_write_options(
                        "a.txt",
                        OpenWriteOptions::new().create().append().truncate(),
                    )
                    .err()
                    .unwrap();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::InvalidInput));
                assert!(
                    fs.open_write_options(
                        "a.txt",
                        OpenWriteOptions::new().create_new().append().truncate()
                    )
                    .is_ok()
                );
            }

            pub(crate) fn open_write_options_create_should_follow_dangling_symlink(fs: &impl Fs) {
                fs.create_symlink_file("link", "a.txt").unwrap();

                fs.write("link", "a").unwrap();

                assert_eq!(fs.read_to_string("a.txt").unwrap(), "a");
                assert!(fs.symlink_metadata("link").unwrap().file_type.is_symlink());
            }

            pub(crate) fn open_write_options_create_with_separator_should_fail(fs: &impl Fs) {
                for (path, kind) in [
                    ("new/", std::io::ErrorKind::IsADirectory),
                    ("x/new/", std::io::ErrorKind::NotFound),
                ] {
                    let error = fs
                        .open_write_options(path, OpenWriteOptions::new().create())
                        .err()
                        .unwrap();

                    assert_eq!(io_kind(&error), Some(kind), "path: {path}");
                }
                assert!(!fs.exists("new").unwrap());
            }

            #[cfg(unix)]
            pub(crate) fn open_write_options_mode_should_set_mode_of_new_file(fs: &impl Fs) {
                fs.open_write_options("a.txt", OpenWriteOptions::new().create().mode(0o600))
                    .unwrap();

                assert_eq!(fs.metadata("a.txt").unwrap().mode, 0o600);
            }

            pub(crate) fn open_write_options_invalid_target_should_fail(fs: &impl Fs) {
                fs.create_dir("dir").unwrap();
                fs.write("a.txt", "").unwrap();

                for (path, kind) in [
                    ("dir", std::io::ErrorKind::IsADirectory),
                    ("x/a.txt", std::io::ErrorKind::NotFound),
                    ("a.txt/a", std::io::ErrorKind::NotADirectory),
                ] {
                    assert_eq!(
                        fs.open_write(path)
                            .map(|_| ())
                            .map_err(|error| io_kind(&error)),
                        Err(Some(kind)),
                        "path: {path}"
                    );
                }
            }
        }

        pub(crate) mod read {
            use super::*;

            pub(crate) fn read_should_return_content(fs: &impl Fs) {
                fs.write("a.bin", [1, 2, 3]).unwrap();

                assert_eq!(fs.read("a.bin").unwrap(), [1, 2, 3]);
            }

            pub(crate) fn read_missing_file_should_fail_with_path(
                fs: &impl Fs,
                current_dir: &Path,
            ) {
                let error = fs.read("missing").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
                assert!(
                    error
                        .description()
                        .ends_with(&format!(": {}", current_dir.join("missing").display())),
                    "{error}"
                );
            }

            pub(crate) fn read_directory_should_fail(fs: &impl Fs) {
                fs.create_dir("dir").unwrap();

                let error = fs.read("dir").unwrap_err();

                #[cfg(unix)]
                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::IsADirectory));
                #[cfg(windows)]
                assert!(io_kind(&error).is_some());
            }
        }

        pub(crate) mod read_to_string {
            use super::*;

            pub(crate) fn read_to_string_should_return_content(fs: &impl Fs) {
                fs.write("a.txt", "content").unwrap();

                assert_eq!(fs.read_to_string("a.txt").unwrap(), "content");
            }

            pub(crate) fn read_to_string_invalid_utf8_should_fail_as_invalid_data(fs: &impl Fs) {
                fs.write("b.bin", [0xff]).unwrap();

                assert_eq!(
                    io_kind(&fs.read_to_string("b.bin").unwrap_err()),
                    Some(std::io::ErrorKind::InvalidData)
                );
                assert_eq!(
                    io_kind(&fs.read_to_string("missing").unwrap_err()),
                    Some(std::io::ErrorKind::NotFound)
                );
            }
        }

        pub(crate) mod write {
            use super::*;

            pub(crate) fn write_should_replace_content(fs: &impl Fs) {
                fs.write("a.txt", "hello").unwrap();

                fs.write("a.txt", "hi").unwrap();

                assert_eq!(fs.read_to_string("a.txt").unwrap(), "hi");
            }

            #[cfg(unix)]
            pub(crate) fn write_should_write_file_that_cannot_be_read(fs: &impl Fs) {
                fs.write("a.txt", "old").unwrap();
                fs.set_mode("a.txt", 0o200).unwrap();

                fs.write("a.txt", "new").unwrap();

                fs.set_mode("a.txt", 0o600).unwrap();
                assert_eq!(fs.read_to_string("a.txt").unwrap(), "new");
            }

            pub(crate) fn write_missing_parent_should_fail(fs: &impl Fs) {
                let error = fs.write("missing/a.txt", "").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
            }

            pub(crate) fn write_directory_should_fail(fs: &impl Fs) {
                fs.create_dir("dir").unwrap();

                let error = fs.write("dir", "content").unwrap_err();

                #[cfg(unix)]
                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::IsADirectory));
                #[cfg(windows)]
                assert!(io_kind(&error).is_some());
                assert!(fs.metadata("dir").unwrap().file_type.is_dir());
            }

            pub(crate) fn write_should_update_modified_time(fs: &impl Fs) {
                fs.write("a.txt", "old").unwrap();
                fs.set_modified("a.txt", SystemTime::UNIX_EPOCH).unwrap();

                fs.write("a.txt", "new").unwrap();

                assert!(fs.metadata("a.txt").unwrap().modified > SystemTime::UNIX_EPOCH);
            }

            pub(crate) fn write_new_file_should_update_modified_time_of_parent(fs: &impl Fs) {
                fs.create_dir("dir").unwrap();
                fs.set_modified("dir", SystemTime::UNIX_EPOCH).unwrap();

                fs.write("dir/a.txt", "").unwrap();

                assert!(fs.metadata("dir").unwrap().modified > SystemTime::UNIX_EPOCH);
            }

            pub(crate) fn write_path_with_nul_byte_should_fail(fs: &impl Fs) {
                let error = fs.write("a\0b", "").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::InvalidInput));
                assert!(!fs.exists("a").unwrap());
            }

            #[cfg(unix)]
            pub(crate) fn write_file_with_trailing_current_dir_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "old").unwrap();

                let error = fs.write("a.txt/.", "new").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotADirectory));
                assert_eq!(fs.read_to_string("a.txt").unwrap(), "old");
            }
        }

        pub(crate) mod read_dir {
            use super::*;

            pub(crate) fn read_dir_should_list_entries_under_given_path(
                fs: &impl Fs,
                current_dir: &Path,
            ) {
                // GIVEN a directory with each kind of entry, linked by a
                // symlink
                fs.create_dir_all("dir/sub").unwrap();
                fs.write("dir/a.txt", "").unwrap();
                fs.create_symlink_file("dir/link", "missing").unwrap();
                fs.create_symlink_dir("dir_link", "dir").unwrap();

                // WHEN it is read through the symlink
                let mut entries: Vec<DirEntry> = fs
                    .read_dir("dir_link")
                    .unwrap()
                    .collect::<Result<_>>()
                    .unwrap();
                entries.sort_by(|a, b| a.path().cmp(b.path()));

                // THEN the entries are listed under the symlink with their
                // own types
                let dir = current_dir.join("dir_link");
                assert_eq!(
                    entries,
                    [
                        DirEntry::new(dir.join("a.txt"), FileType::File),
                        DirEntry::new(dir.join("link"), FileType::Symlink),
                        DirEntry::new(dir.join("sub"), FileType::Dir),
                    ]
                );
            }

            pub(crate) fn read_dir_invalid_path_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();
                fs.create_symlink_file("link", "missing").unwrap();

                for (path, kind) in [
                    ("a.txt", std::io::ErrorKind::NotADirectory),
                    ("link", std::io::ErrorKind::NotFound),
                    ("missing", std::io::ErrorKind::NotFound),
                ] {
                    assert_eq!(
                        fs.read_dir(path)
                            .map(|_| ())
                            .map_err(|error| io_kind(&error)),
                        Err(Some(kind)),
                        "path: {path}"
                    );
                }
            }
        }

        pub(crate) mod create_dir {
            use super::*;

            pub(crate) fn create_dir_should_create_only_new_dir(fs: &impl Fs) {
                fs.create_dir("dir/").unwrap();

                assert!(fs.metadata("dir").unwrap().file_type.is_dir());
                assert_eq!(
                    io_kind(&fs.create_dir("dir").unwrap_err()),
                    Some(std::io::ErrorKind::AlreadyExists)
                );
            }

            pub(crate) fn create_dir_invalid_path_should_fail(fs: &impl Fs) {
                fs.create_symlink_file("link", "missing").unwrap();
                fs.write("a.txt", "").unwrap();

                for (path, kind) in [
                    ("link", std::io::ErrorKind::AlreadyExists),
                    ("x/dir", std::io::ErrorKind::NotFound),
                    ("a.txt/dir", std::io::ErrorKind::NotADirectory),
                ] {
                    assert_eq!(
                        fs.create_dir(path).map_err(|error| io_kind(&error)),
                        Err(Some(kind)),
                        "path: {path}"
                    );
                }
                assert!(!fs.exists("missing").unwrap());
            }

            #[cfg(unix)]
            pub(crate) fn create_dir_with_trailing_current_dir_should_fail(fs: &impl Fs) {
                fs.create_dir("dir").unwrap();

                let error = fs.create_dir("missing/.").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
                assert!(!fs.exists("missing").unwrap());
                assert_eq!(
                    io_kind(&fs.create_dir("dir/.").unwrap_err()),
                    Some(std::io::ErrorKind::AlreadyExists)
                );
            }
        }

        pub(crate) mod create_dir_all {
            use super::*;

            pub(crate) fn create_dir_all_should_create_missing_parents(fs: &impl Fs) {
                fs.create_dir("a").unwrap();
                fs.create_symlink_dir("link", "a").unwrap();

                fs.create_dir_all("link/b/../c/d").unwrap();
                fs.create_dir_all("link/c/d").unwrap();

                // WHY: Windows removes `b/..` before creating anything.
                assert_eq!(fs.exists("a/b").unwrap(), cfg!(unix));
                assert!(fs.metadata("a/c/d").unwrap().file_type.is_dir());
                assert!(fs.symlink_metadata("link").unwrap().file_type.is_symlink());
            }

            pub(crate) fn create_dir_all_existing_non_dir_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();
                fs.create_symlink_file("link", "missing").unwrap();

                for (path, kind) in [
                    ("a.txt", std::io::ErrorKind::AlreadyExists),
                    ("a.txt/b", std::io::ErrorKind::NotADirectory),
                    ("link/b", std::io::ErrorKind::AlreadyExists),
                ] {
                    assert_eq!(
                        fs.create_dir_all(path).map_err(|error| io_kind(&error)),
                        Err(Some(kind)),
                        "path: {path}"
                    );
                }
                assert!(!fs.exists("missing").unwrap());
            }
        }

        pub(crate) mod exists {
            use super::*;

            pub(crate) fn exists_should_follow_symlinks(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();
                fs.create_symlink_file("link", "a.txt").unwrap();
                fs.create_symlink_file("dangling", "missing").unwrap();

                assert!(fs.exists("a.txt").unwrap());
                assert!(fs.exists("link").unwrap());
                assert!(!fs.exists("dangling").unwrap());
                assert!(!fs.exists("missing").unwrap());
            }

            #[cfg(unix)]
            pub(crate) fn exists_file_as_parent_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();

                let error = fs.exists("a.txt/b").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotADirectory));
            }

            #[cfg(windows)]
            pub(crate) fn exists_file_as_parent_should_not_exist(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();

                assert!(!fs.exists("a.txt/b").unwrap());
            }
        }

        pub(crate) mod metadata {
            use super::*;

            pub(crate) fn metadata_should_follow_symlinks(fs: &impl Fs) {
                fs.write(".a.txt", "content").unwrap();
                fs.create_symlink_file("link", ".a.txt").unwrap();

                let metadata = fs.metadata("link").unwrap();

                assert_eq!(metadata.file_type, FileType::File);
                assert_eq!(metadata.len, 7);
                #[cfg(unix)]
                {
                    assert!(!metadata.hidden);
                    assert!(fs.metadata(".a.txt").unwrap().hidden);
                }
            }

            pub(crate) fn metadata_missing_path_should_fail(fs: &impl Fs) {
                fs.create_symlink_file("link", "missing").unwrap();

                for path in ["missing", "link"] {
                    assert_eq!(
                        fs.metadata(path).map_err(|error| io_kind(&error)),
                        Err(Some(std::io::ErrorKind::NotFound)),
                        "path: {path}"
                    );
                }
            }

            #[cfg(unix)]
            pub(crate) fn metadata_file_with_trailing_current_dir_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();

                let error = fs.metadata("a.txt/.").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotADirectory));
            }
        }

        pub(crate) mod symlink_metadata {
            use super::*;

            pub(crate) fn symlink_metadata_should_not_follow_symlink(fs: &impl Fs) {
                fs.create_symlink_file("link", "missing").unwrap();

                let metadata = fs.symlink_metadata("link").unwrap();

                assert_eq!(metadata.file_type, FileType::Symlink);
                #[cfg(unix)]
                assert_eq!((metadata.len, metadata.mode), (7, 0o777));
                assert_eq!(
                    io_kind(&fs.symlink_metadata("missing").unwrap_err()),
                    Some(std::io::ErrorKind::NotFound)
                );
            }
        }

        pub(crate) mod create_symlink_file {
            use super::*;

            pub(crate) fn create_symlink_file_should_resolve_relative_target_against_link_parent(
                fs: &impl Fs,
            ) {
                fs.create_dir("dir").unwrap();
                fs.write("dir/a.txt", "content").unwrap();

                fs.create_symlink_file("dir/link", "a.txt").unwrap();

                assert_eq!(fs.read_to_string("dir/link").unwrap(), "content");
                assert_eq!(fs.read_link("dir/link").unwrap(), Path::new("a.txt"));
            }

            pub(crate) fn create_symlink_file_with_parent_dir_in_target_should_resolve_per_os(
                fs: &impl Fs,
            ) {
                fs.write("a.txt", "content").unwrap();

                fs.create_symlink_file("link", "missing/../a.txt").unwrap();

                #[cfg(unix)]
                assert_eq!(
                    io_kind(&fs.read_to_string("link").unwrap_err()),
                    Some(std::io::ErrorKind::NotFound)
                );
                // WHY: Windows removes `..` from the target lexically.
                #[cfg(windows)]
                assert_eq!(fs.read_to_string("link").unwrap(), "content");
            }

            pub(crate) fn create_symlink_file_current_dir_target_should_resolve(fs: &impl Fs) {
                fs.write("a.txt", "a").unwrap();

                fs.create_symlink_file("link", "./a.txt").unwrap();

                assert_eq!(fs.read_to_string("link").unwrap(), "a");
            }

            pub(crate) fn create_symlink_file_should_allow_dangling_target(fs: &impl Fs) {
                fs.create_symlink_file("link", "missing").unwrap();

                assert!(!fs.exists("link").unwrap());
                fs.write("missing", "content").unwrap();
                assert_eq!(fs.read_to_string("link").unwrap(), "content");
            }

            pub(crate) fn create_symlink_file_existing_path_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();
                fs.create_symlink_file("link", "missing").unwrap();

                for path in ["a.txt", "link"] {
                    assert_eq!(
                        fs.create_symlink_file(path, "other")
                            .map_err(|error| io_kind(&error)),
                        Err(Some(std::io::ErrorKind::AlreadyExists)),
                        "path: {path}"
                    );
                }
                assert_eq!(fs.read_link("link").unwrap(), Path::new("missing"));
            }

            pub(crate) fn create_symlink_file_invalid_path_should_fail(fs: &impl Fs) {
                #[cfg_attr(not(unix), allow(clippy::single_element_loop))]
                for (path, target) in [
                    ("x/link", "a"),
                    #[cfg(unix)]
                    ("link/", "a"),
                    #[cfg(unix)]
                    ("link", ""),
                ] {
                    assert_eq!(
                        fs.create_symlink_file(path, target)
                            .map_err(|error| io_kind(&error)),
                        Err(Some(std::io::ErrorKind::NotFound)),
                        "path: {path}, target: {target}"
                    );
                }
                assert!(fs.symlink_metadata("link").is_err());
            }
        }

        pub(crate) mod create_symlink_dir {
            use super::*;

            pub(crate) fn create_symlink_dir_should_link_dir_created_later(fs: &impl Fs) {
                fs.create_symlink_dir("link", "dir").unwrap();
                fs.create_dir("dir").unwrap();

                assert!(fs.metadata("link").unwrap().file_type.is_dir());
                assert_eq!(
                    io_kind(&fs.create_symlink_dir("link", "dir").unwrap_err()),
                    Some(std::io::ErrorKind::AlreadyExists)
                );
            }
        }

        pub(crate) mod read_link {
            use super::*;

            pub(crate) fn read_link_should_return_target_as_stored(fs: &impl Fs) {
                fs.create_symlink_file("link", "../a/./b").unwrap();

                assert_eq!(fs.read_link("link").unwrap(), Path::new("../a/./b"));
            }

            pub(crate) fn read_link_not_symlink_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();
                fs.create_dir("dir").unwrap();

                for path in ["a.txt", "dir"] {
                    let error = fs.read_link(path).expect_err(path);

                    #[cfg(unix)]
                    assert_eq!(
                        io_kind(&error),
                        Some(std::io::ErrorKind::InvalidInput),
                        "path: {path}"
                    );
                    // WHY: ERROR_NOT_A_REPARSE_POINT, which has no stable kind.
                    #[cfg(windows)]
                    assert_eq!(
                        error
                            .source_as::<std::io::Error>()
                            .and_then(std::io::Error::raw_os_error),
                        Some(4390),
                        "path: {path}"
                    );
                }
            }

            pub(crate) fn read_link_missing_path_should_fail(fs: &impl Fs) {
                assert_eq!(
                    io_kind(&fs.read_link("missing").unwrap_err()),
                    Some(std::io::ErrorKind::NotFound)
                );
            }
        }

        pub(crate) mod canonicalize {
            use super::*;

            pub(crate) fn canonicalize_should_resolve_symlinks_and_parent_dirs(
                fs: &impl Fs,
                canonical_current_dir: &Path,
            ) {
                fs.create_dir_all("dir/sub").unwrap();
                fs.create_symlink_dir("link", "dir/sub").unwrap();
                let dir = canonical_current_dir.join("dir");
                // WHY: Unix follows the symlink before `..`, Windows
                // removes `..` lexically first.
                #[cfg(unix)]
                let parent = "link/../sub";
                #[cfg(windows)]
                let parent = "link/../dir/sub";

                assert_eq!(fs.canonicalize("link").unwrap(), dir.join("sub"));
                assert_eq!(fs.canonicalize(parent).unwrap(), dir.join("sub"));
            }

            pub(crate) fn canonicalize_missing_path_should_fail(fs: &impl Fs) {
                fs.create_symlink_file("link", "missing").unwrap();

                for path in ["missing", "link"] {
                    assert_eq!(
                        fs.canonicalize(path).map_err(|error| io_kind(&error)),
                        Err(Some(std::io::ErrorKind::NotFound)),
                        "path: {path}"
                    );
                }
            }
        }

        pub(crate) mod create_hard_link {
            use super::*;

            pub(crate) fn create_hard_link_should_share_content(fs: &impl Fs) {
                fs.write("a.txt", "old").unwrap();

                fs.create_hard_link("b.txt", "a.txt").unwrap();
                fs.write("b.txt", "new").unwrap();

                assert_eq!(fs.read_to_string("a.txt").unwrap(), "new");
            }

            pub(crate) fn create_hard_link_to_symlink_should_link_symlink_itself(fs: &impl Fs) {
                fs.write("a.txt", "a").unwrap();
                fs.create_symlink_file("link", "a.txt").unwrap();

                fs.create_hard_link("other", "link").unwrap();
                fs.remove_file("link").unwrap();

                assert_eq!(fs.read_link("other").unwrap(), Path::new("a.txt"));
                assert_eq!(fs.read_to_string("other").unwrap(), "a");
            }

            pub(crate) fn create_hard_link_invalid_path_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();
                fs.create_dir("dir").unwrap();

                for (path, target, kind) in [
                    ("a.txt", "a.txt", std::io::ErrorKind::AlreadyExists),
                    ("x/b.txt", "a.txt", std::io::ErrorKind::NotFound),
                    #[cfg(unix)]
                    ("b/", "a.txt", std::io::ErrorKind::NotFound),
                    ("b.txt", "dir", std::io::ErrorKind::PermissionDenied),
                    ("b.txt", "missing", std::io::ErrorKind::NotFound),
                ] {
                    assert_eq!(
                        fs.create_hard_link(path, target)
                            .map_err(|error| io_kind(&error)),
                        Err(Some(kind)),
                        "path: {path}, target: {target}"
                    );
                }
                assert!(!fs.exists("b.txt").unwrap());
            }
        }

        pub(crate) mod copy {
            use super::*;

            pub(crate) fn copy_should_overwrite_target(fs: &impl Fs) {
                fs.write("a.txt", "new").unwrap();
                fs.write("b.txt", "old content").unwrap();

                assert_eq!(fs.copy("a.txt", "b.txt").unwrap(), 3);

                assert_eq!(fs.read_to_string("b.txt").unwrap(), "new");
            }

            pub(crate) fn copy_should_copy_content_and_permissions_through_symlinks(fs: &impl Fs) {
                // GIVEN a file with restricted permissions linked by a symlink
                fs.write("a.txt", "content").unwrap();
                fs.create_symlink_file("link", "a.txt").unwrap();
                #[cfg(unix)]
                fs.set_mode("a.txt", 0o640).unwrap();

                // WHEN it is copied through the symlink
                let copied = fs.copy("link", "b.txt").unwrap();

                // THEN an independent copy of the file itself is made
                assert_eq!(copied, 7);
                assert!(fs.symlink_metadata("b.txt").unwrap().file_type.is_file());
                #[cfg(unix)]
                assert_eq!(fs.metadata("b.txt").unwrap().mode, 0o640);
                fs.write("b.txt", "other").unwrap();
                assert_eq!(fs.read_to_string("a.txt").unwrap(), "content");
            }

            pub(crate) fn copy_should_follow_dangling_symlink_target(fs: &impl Fs) {
                fs.write("a.txt", "content").unwrap();
                fs.create_symlink_file("link", "b.txt").unwrap();

                fs.copy("a.txt", "link").unwrap();

                assert_eq!(fs.read_to_string("b.txt").unwrap(), "content");
                assert!(fs.symlink_metadata("link").unwrap().file_type.is_symlink());
            }

            pub(crate) fn copy_onto_itself_should_fail_keeping_content(fs: &impl Fs) {
                fs.write("a.txt", "content").unwrap();
                fs.create_hard_link("hard", "a.txt").unwrap();
                fs.create_symlink_file("link", "a.txt").unwrap();

                for target in ["a.txt", "./a.txt", "hard", "link"] {
                    assert!(fs.copy("a.txt", target).is_err(), "{target}");
                    assert_eq!(fs.read_to_string("a.txt").unwrap(), "content", "{target}");
                }
            }

            pub(crate) fn copy_dir_should_fail(fs: &impl Fs) {
                fs.create_dir("dir").unwrap();

                let error = fs.copy("dir", "copy").unwrap_err();

                #[cfg(unix)]
                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::InvalidInput));
                #[cfg(windows)]
                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::PermissionDenied));
                assert!(!fs.exists("copy").unwrap());
            }

            pub(crate) fn copy_missing_source_should_fail(fs: &impl Fs) {
                let error = fs.copy("missing", "b.txt").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
                assert!(!fs.exists("b.txt").unwrap());
            }

            pub(crate) fn copy_invalid_target_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "a").unwrap();
                fs.create_dir("dir").unwrap();

                for (target, kind) in [
                    ("dir", std::io::ErrorKind::IsADirectory),
                    ("x/b", std::io::ErrorKind::NotFound),
                    ("new/", std::io::ErrorKind::IsADirectory),
                    ("x/new/", std::io::ErrorKind::NotFound),
                ] {
                    assert_eq!(
                        fs.copy("a.txt", target).map_err(|error| io_kind(&error)),
                        Err(Some(kind)),
                        "target: {target}"
                    );
                }
                assert!(!fs.exists("new").unwrap());
            }
        }

        pub(crate) mod copy_dir_all {
            use super::*;

            pub(crate) fn copy_dir_all_should_copy_tree_with_symlinks_as_symlinks(fs: &impl Fs) {
                fs.create_dir_all("dir/sub").unwrap();
                fs.write("dir/sub/a.txt", "content").unwrap();
                fs.create_symlink_file("dir/link", "sub/a.txt").unwrap();
                fs.create_symlink_file("dir/dangling", "missing").unwrap();

                fs.copy_dir_all("dir", "dir/copy").unwrap();

                assert_eq!(fs.read_to_string("dir/copy/sub/a.txt").unwrap(), "content");
                assert_eq!(
                    fs.read_link("dir/copy/link").unwrap(),
                    Path::new("sub/a.txt")
                );
                assert_eq!(fs.read_to_string("dir/copy/link").unwrap(), "content");
                assert_eq!(
                    fs.read_link("dir/copy/dangling").unwrap(),
                    Path::new("missing")
                );
                assert!(!fs.exists("dir/copy/copy").unwrap());
            }

            #[cfg(unix)]
            pub(crate) fn copy_dir_all_should_copy_dir_permissions(fs: &impl Fs) {
                fs.create_dir_all("dir/sub").unwrap();
                fs.set_mode("dir/sub", 0o500).unwrap();

                fs.copy_dir_all("dir", "copy").unwrap();

                assert_eq!(fs.metadata("copy/sub").unwrap().mode, 0o500);
            }

            pub(crate) fn copy_dir_all_invalid_path_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();
                fs.create_dir("dir").unwrap();
                fs.create_dir("copy").unwrap();

                for (source, target, kind) in [
                    ("a.txt", "b", std::io::ErrorKind::NotADirectory),
                    ("missing", "b", std::io::ErrorKind::NotFound),
                    ("dir", "copy", std::io::ErrorKind::AlreadyExists),
                ] {
                    assert_eq!(
                        fs.copy_dir_all(source, target)
                            .map_err(|error| io_kind(&error)),
                        Err(Some(kind)),
                        "source: {source}, target: {target}"
                    );
                }
                assert!(!fs.exists("b").unwrap());
            }
        }

        pub(crate) mod remove_file {
            use super::*;

            pub(crate) fn remove_file_should_remove_file_or_symlink_only(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();
                fs.write("b.txt", "").unwrap();
                fs.create_dir("dir").unwrap();
                fs.create_symlink_file("link", "b.txt").unwrap();
                fs.create_symlink_dir("dir_link", "dir").unwrap();

                fs.remove_file("a.txt").unwrap();
                fs.remove_file("link").unwrap();
                fs.remove_file("dir_link").unwrap();

                assert!(!fs.exists("a.txt").unwrap());
                assert!(fs.symlink_metadata("link").is_err());
                assert!(fs.symlink_metadata("dir_link").is_err());
                assert!(fs.exists("b.txt").unwrap());
                assert!(fs.exists("dir").unwrap());
            }

            pub(crate) fn remove_file_invalid_path_should_fail(fs: &impl Fs) {
                fs.create_dir("dir").unwrap();
                fs.write("a.txt", "").unwrap();

                for (path, kind) in [
                    ("missing", std::io::ErrorKind::NotFound),
                    ("dir", std::io::ErrorKind::IsADirectory),
                    ("a.txt/", std::io::ErrorKind::NotADirectory),
                ] {
                    assert_eq!(
                        fs.remove_file(path).map_err(|error| io_kind(&error)),
                        Err(Some(kind)),
                        "path: {path}"
                    );
                }
                assert!(fs.exists("dir").unwrap());
                assert!(fs.exists("a.txt").unwrap());
            }

            #[cfg(unix)]
            pub(crate) fn remove_file_symlink_to_dir_with_trailing_separator_should_fail(
                fs: &impl Fs,
            ) {
                fs.create_dir("dir").unwrap();
                fs.create_symlink_dir("link", "dir").unwrap();

                let error = fs.remove_file("link/").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotADirectory));
                assert!(fs.exists("link").unwrap());
            }
        }

        pub(crate) mod remove_dir {
            use super::*;

            pub(crate) fn remove_dir_should_remove_only_empty_dir(fs: &impl Fs) {
                fs.create_dir_all("dir/sub").unwrap();

                assert_eq!(
                    io_kind(&fs.remove_dir("dir").unwrap_err()),
                    Some(std::io::ErrorKind::DirectoryNotEmpty)
                );
                fs.remove_dir("dir/sub").unwrap();
                fs.remove_dir("dir").unwrap();
                assert!(!fs.exists("dir").unwrap());
            }

            pub(crate) fn remove_dir_invalid_path_should_fail(fs: &impl Fs) {
                fs.create_dir("dir").unwrap();
                fs.write("a.txt", "").unwrap();
                fs.create_symlink_file("link", "dir").unwrap();

                for (path, kind) in [
                    ("a.txt", std::io::ErrorKind::NotADirectory),
                    #[cfg(unix)]
                    ("link", std::io::ErrorKind::NotADirectory),
                    ("missing", std::io::ErrorKind::NotFound),
                    #[cfg(unix)]
                    ("/", std::io::ErrorKind::ResourceBusy),
                ] {
                    assert_eq!(
                        fs.remove_dir(path).map_err(|error| io_kind(&error)),
                        Err(Some(kind)),
                        "path: {path}"
                    );
                }
                assert!(fs.symlink_metadata("link").is_ok());
            }

            #[cfg(unix)]
            pub(crate) fn remove_dir_symlink_to_dir_with_trailing_separator_should_fail(
                fs: &impl Fs,
            ) {
                fs.create_dir("dir").unwrap();
                fs.create_symlink_dir("link", "dir").unwrap();

                let error = fs.remove_dir("link/").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotADirectory));
                assert!(fs.exists("dir").unwrap());
                assert!(fs.exists("link").unwrap());
            }
        }

        pub(crate) mod remove_dir_all {
            use super::*;

            pub(crate) fn remove_dir_all_should_remove_tree_but_not_symlink_targets(fs: &impl Fs) {
                fs.create_dir_all("dir/sub").unwrap();
                fs.write("dir/sub/a.txt", "").unwrap();
                fs.write("b.txt", "").unwrap();
                fs.create_symlink_file("dir/link", "../b.txt").unwrap();
                fs.create_symlink_file("link", "b.txt").unwrap();

                fs.remove_dir_all("dir").unwrap();
                fs.remove_dir_all("link").unwrap();

                assert!(!fs.exists("dir").unwrap());
                assert!(fs.symlink_metadata("link").is_err());
                assert!(fs.exists("b.txt").unwrap());
            }

            pub(crate) fn remove_dir_all_root_should_fail_without_removing_anything(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();

                let error = fs.remove_dir_all("/").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::ResourceBusy));
                assert!(fs.exists("a.txt").unwrap());
            }

            pub(crate) fn remove_dir_all_trailing_separator_should_remove_dir(fs: &impl Fs) {
                fs.create_dir_all("dir/sub").unwrap();

                fs.remove_dir_all("dir/").unwrap();

                assert!(!fs.exists("dir").unwrap());
            }

            pub(crate) fn remove_dir_all_symlink_with_trailing_separator_should_fail_without_removing_target(
                fs: &impl Fs,
            ) {
                fs.create_dir("dir").unwrap();
                fs.write("dir/a.txt", "").unwrap();
                fs.create_symlink_dir("link", "dir").unwrap();

                let error = fs.remove_dir_all("link/").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotADirectory));
                assert!(fs.exists("dir/a.txt").unwrap());
            }

            pub(crate) fn remove_dir_all_missing_path_with_trailing_separator_should_fail(
                fs: &impl Fs,
            ) {
                let error = fs.remove_dir_all("missing/").unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
            }

            pub(crate) fn remove_dir_all_invalid_path_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();

                for (path, kind) in [
                    ("a.txt", std::io::ErrorKind::NotADirectory),
                    ("missing", std::io::ErrorKind::NotFound),
                ] {
                    assert_eq!(
                        fs.remove_dir_all(path).map_err(|error| io_kind(&error)),
                        Err(Some(kind)),
                        "path: {path}"
                    );
                }
                assert!(fs.exists("a.txt").unwrap());
            }
        }

        pub(crate) mod rename {
            use super::*;

            pub(crate) fn rename_should_replace_target_atomically(fs: &impl Fs) {
                fs.write("a.txt.tmp", "new").unwrap();
                fs.write("a.txt", "old").unwrap();
                fs.write("b.txt", "b").unwrap();
                fs.create_symlink_file("link", "b.txt").unwrap();

                fs.rename("a.txt.tmp", "a.txt").unwrap();
                fs.rename("a.txt", "link").unwrap();

                assert!(!fs.exists("a.txt.tmp").unwrap());
                assert!(!fs.exists("a.txt").unwrap());
                assert!(fs.symlink_metadata("link").unwrap().file_type.is_file());
                assert_eq!(fs.read_to_string("link").unwrap(), "new");
                assert_eq!(fs.read_to_string("b.txt").unwrap(), "b");
            }

            pub(crate) fn rename_should_move_tree(fs: &impl Fs) {
                fs.create_dir_all("a/sub").unwrap();
                fs.write("a/sub/f.txt", "f").unwrap();
                fs.create_dir("b").unwrap();

                fs.rename("a", "b").unwrap();

                assert_eq!(fs.read_to_string("b/sub/f.txt").unwrap(), "f");
                assert!(!fs.exists("a").unwrap());
            }

            pub(crate) fn rename_to_new_path_should_move_entry(fs: &impl Fs) {
                fs.create_dir("dir").unwrap();
                fs.write("a.txt", "a").unwrap();
                fs.create_symlink_file("link", "a.txt").unwrap();

                for (source, target) in [("a.txt", "dir/b.txt"), ("link", "dir/link")] {
                    fs.rename(source, target).unwrap();

                    assert!(fs.symlink_metadata(source).is_err(), "source: {source}");
                }
                assert_eq!(fs.read_to_string("dir/b.txt").unwrap(), "a");
                assert_eq!(fs.read_link("dir/link").unwrap(), Path::new("a.txt"));
            }

            pub(crate) fn rename_same_entry_should_do_nothing(fs: &impl Fs) {
                fs.write("a.txt", "a").unwrap();
                fs.create_dir("dir").unwrap();

                fs.rename("a.txt", "a.txt").unwrap();
                fs.rename("dir", "dir/").unwrap();

                assert_eq!(fs.read_to_string("a.txt").unwrap(), "a");
                assert!(fs.exists("dir").unwrap());
            }

            pub(crate) fn rename_hard_link_onto_same_file_should_do_nothing(fs: &impl Fs) {
                fs.write("a.txt", "a").unwrap();
                fs.create_hard_link("b.txt", "a.txt").unwrap();

                fs.rename("a.txt", "b.txt").unwrap();

                // WHY: POSIX `rename` does nothing for two links of the same
                // file, Windows removes the source.
                assert_eq!(fs.exists("a.txt").unwrap(), cfg!(unix));
                assert_eq!(fs.read_to_string("b.txt").unwrap(), "a");
            }

            pub(crate) fn rename_invalid_source_should_fail(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();
                fs.create_dir("dir").unwrap();
                fs.create_symlink_dir("link", "dir").unwrap();

                for (source, kind) in [
                    ("missing", std::io::ErrorKind::NotFound),
                    ("a.txt/", std::io::ErrorKind::NotADirectory),
                    ("link/", std::io::ErrorKind::NotADirectory),
                ] {
                    assert_eq!(
                        fs.rename(source, "b").map_err(|error| io_kind(&error)),
                        Err(Some(kind)),
                        "source: {source}"
                    );
                }
                assert!(!fs.exists("b").unwrap());
            }

            pub(crate) fn rename_invalid_target_should_fail(fs: &impl Fs) {
                fs.create_dir_all("dir/sub").unwrap();
                fs.create_dir_all("full/x").unwrap();
                fs.create_dir("empty").unwrap();
                fs.write("a.txt", "").unwrap();

                for (source, target, kind) in [
                    ("a.txt", "x/b", std::io::ErrorKind::NotFound),
                    ("dir", "dir/sub/x", std::io::ErrorKind::InvalidInput),
                    ("empty", "full", std::io::ErrorKind::DirectoryNotEmpty),
                    ("empty", "a.txt", std::io::ErrorKind::NotADirectory),
                    ("a.txt", "empty", std::io::ErrorKind::IsADirectory),
                ] {
                    assert_eq!(
                        fs.rename(source, target).map_err(|error| io_kind(&error)),
                        Err(Some(kind)),
                        "source: {source}, target: {target}"
                    );
                }
                assert!(fs.exists("dir/sub").unwrap());
                assert!(fs.exists("full/x").unwrap());
                assert!(fs.exists("a.txt").unwrap());
            }

            pub(crate) fn rename_target_with_separator_should_require_dir(fs: &impl Fs) {
                fs.create_dir("dir").unwrap();
                fs.create_dir("other").unwrap();
                fs.write("a.txt", "").unwrap();
                fs.create_symlink_dir("link", "other").unwrap();

                for (source, target) in [
                    ("dir", "a.txt/"),
                    ("a.txt", "new/"),
                    ("a.txt", "other/"),
                    ("a.txt", "link/"),
                ] {
                    assert_eq!(
                        fs.rename(source, target).map_err(|error| io_kind(&error)),
                        Err(Some(std::io::ErrorKind::NotADirectory)),
                        "source: {source}, target: {target}"
                    );
                }
                fs.rename("dir", "new/").unwrap();
                assert!(fs.exists("new").unwrap());
            }
        }

        pub(crate) mod set_readonly {
            use super::*;

            #[cfg(unix)]
            pub(crate) fn set_readonly_should_map_to_write_bits(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();

                for (initial, readonly, expected) in [
                    (0o644, true, 0o444),
                    (0o444, true, 0o444),
                    (0o444, false, 0o644),
                    (0o664, false, 0o664),
                ] {
                    fs.set_mode("a.txt", initial).unwrap();

                    fs.set_readonly("a.txt", readonly).unwrap();

                    let metadata = fs.metadata("a.txt").unwrap();
                    assert_eq!(
                        (metadata.mode, metadata.readonly),
                        (expected, readonly),
                        "initial: {initial:o}, readonly: {readonly}"
                    );
                }
            }

            #[cfg(windows)]
            pub(crate) fn set_readonly_should_set_and_clear_attribute(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();

                for readonly in [true, true, false, false] {
                    fs.set_readonly("a.txt", readonly).unwrap();

                    assert_eq!(
                        fs.metadata("a.txt").unwrap().readonly,
                        readonly,
                        "readonly: {readonly}"
                    );
                }
            }

            pub(crate) fn set_readonly_missing_path_should_fail(fs: &impl Fs) {
                let error = fs.set_readonly("missing", true).unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
            }
        }

        #[cfg(unix)]
        pub(crate) mod set_executable {
            use super::*;

            pub(crate) fn set_executable_should_map_to_execute_bits(fs: &impl Fs) {
                fs.write("a.sh", "").unwrap();
                fs.set_mode("a.sh", 0o640).unwrap();

                fs.set_executable("a.sh", true).unwrap();
                assert_eq!(fs.metadata("a.sh").unwrap().mode, 0o750);
                assert!(fs.metadata("a.sh").unwrap().executable);
                fs.set_executable("a.sh", false).unwrap();
                assert_eq!(fs.metadata("a.sh").unwrap().mode, 0o640);
                fs.set_executable("a.sh", false).unwrap();
                assert_eq!(fs.metadata("a.sh").unwrap().mode, 0o640);
            }

            pub(crate) fn set_executable_dir_should_fail(fs: &impl Fs) {
                fs.create_dir("dir").unwrap();
                let initial = fs.metadata("dir").unwrap().mode;

                let error = fs.set_executable("dir", false).unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::IsADirectory));
                assert_eq!(fs.metadata("dir").unwrap().mode, initial);
            }

            pub(crate) fn set_executable_missing_path_should_fail(fs: &impl Fs) {
                let error = fs.set_executable("missing", true).unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
            }
        }

        pub(crate) mod set_modified {
            use super::*;

            pub(crate) fn set_modified_should_set_modification_time(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();
                fs.create_dir("dir").unwrap();

                for path in ["a.txt", "dir"] {
                    fs.set_modified(path, SystemTime::UNIX_EPOCH).unwrap();

                    assert_eq!(
                        fs.metadata(path).unwrap().modified,
                        SystemTime::UNIX_EPOCH,
                        "path: {path}"
                    );
                }
            }

            pub(crate) fn set_modified_missing_path_should_fail(fs: &impl Fs) {
                let error = fs
                    .set_modified("missing", SystemTime::UNIX_EPOCH)
                    .unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
            }
        }

        #[cfg(unix)]
        pub(crate) mod set_mode {
            use super::*;

            pub(crate) fn set_mode_should_set_permission_bits_only(fs: &impl Fs) {
                fs.write("a.txt", "").unwrap();

                fs.set_mode("a.txt", 0o170_640).unwrap();

                assert_eq!(fs.metadata("a.txt").unwrap().mode, 0o640);
            }

            pub(crate) fn set_mode_missing_path_should_fail(fs: &impl Fs) {
                let error = fs.set_mode("missing", 0o644).unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
            }
        }
    }

    /// The contract cases of [`ReadFile`], each taking a `file` holding
    /// `content`, open for reading.
    pub(crate) mod read_file {
        use super::*;

        pub(crate) mod path {
            use super::*;

            pub(crate) fn path_should_return_path_resolved_against_current_dir(
                file: &impl ReadFile,
                expected: &Path,
            ) {
                assert_eq!(file.path(), expected);
            }
        }

        pub(crate) mod metadata {
            use super::*;

            pub(crate) fn metadata_should_return_metadata_of_open_file(
                file: &impl ReadFile,
                expected: &Metadata,
            ) {
                let metadata = file.metadata().unwrap();

                assert_eq!(&metadata, expected);
                assert_eq!((metadata.file_type, metadata.len), (FileType::File, 7));
            }
        }

        pub(crate) mod read_at {
            use super::*;

            pub(crate) fn read_at_should_read_at_offset_from_shared_handle(
                file: &(impl ReadFile + Sync),
            ) {
                // WHEN the file is read at different offsets from two threads
                let (low, high) = std::thread::scope(|scope| {
                    let low = scope.spawn(|| {
                        let mut buf = [0; 3];
                        file.read_at(&mut buf, 1).map(|read| buf[..read].to_vec())
                    });
                    let high = scope.spawn(|| {
                        let mut buf = [0; 3];
                        file.read_at(&mut buf, 5).map(|read| buf[..read].to_vec())
                    });
                    (low.join().unwrap(), high.join().unwrap())
                });

                // THEN each gets the bytes at its offset, short at the end
                assert_eq!(low.unwrap(), b"ont");
                assert_eq!(high.unwrap(), b"nt");
                // AND nothing is read at or beyond the end
                for offset in [7, 100] {
                    assert_eq!(file.read_at(&mut [0; 3], offset).unwrap(), 0, "{offset}");
                }
            }
        }

        pub(crate) mod read_exact_at {
            use super::*;

            pub(crate) fn read_exact_at_should_fill_buffer_or_fail_at_end(file: &impl ReadFile) {
                let mut buf = [0; 4];

                file.read_exact_at(&mut buf, 3).unwrap();

                assert_eq!(&buf, b"tent");
                let error = file.read_exact_at(&mut buf, 4).err().unwrap();
                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::UnexpectedEof));
                assert_eq!(
                    error.description(),
                    format!(
                        "File could not be read, it ends before the buffer is filled: {}",
                        file.path().display()
                    )
                );
            }
        }

        pub(crate) mod lock_shared {
            use super::*;

            pub(crate) fn lock_shared_should_lock_until_unlocked(file: &impl ReadFile) {
                file.lock_shared().unwrap();
                file.unlock().unwrap();

                assert!(file.try_lock_shared().unwrap());
            }
        }

        pub(crate) mod try_lock_shared {
            use super::*;

            pub(crate) fn try_lock_shared_should_lock_unlocked_file(file: &impl ReadFile) {
                assert!(file.try_lock_shared().unwrap());
                assert!(file.try_lock_shared().unwrap());
            }
        }

        pub(crate) mod unlock {
            use super::*;

            pub(crate) fn unlock_should_release_lock(file: &impl ReadFile) {
                file.unlock().unwrap();
                file.lock_shared().unwrap();

                file.unlock().unwrap();

                assert!(file.try_lock_shared().unwrap());
            }
        }
    }

    /// The contract cases of [`WriteFile`], each taking a `file` holding
    /// `content`, open for reading and writing.
    pub(crate) mod write_file {
        use super::*;

        /// Returns the content of the `file`.
        fn content(file: &impl WriteFile) -> Vec<u8> {
            let mut buf = vec![0; file.metadata().unwrap().len as usize];
            file.read_exact_at(&mut buf, 0).unwrap();
            buf
        }

        pub(crate) mod write_at {
            use super::*;

            pub(crate) fn write_at_should_write_at_offset_and_extend_file(file: &impl WriteFile) {
                assert_eq!(file.write_at(b"C", 0).unwrap(), 1);
                assert_eq!(file.write_at(b"!", 8).unwrap(), 1);

                assert_eq!(content(file), b"Content\0!");
            }

            /// Takes a `file` opened for [`append`](OpenWriteOptions::append).
            #[cfg(target_os = "linux")]
            pub(crate) fn write_at_append_should_ignore_offset(file: &impl WriteFile) {
                assert_eq!(file.write_at(b"!", 0).unwrap(), 1);

                assert_eq!(content(file), b"content!");
            }
        }

        pub(crate) mod write_all_at {
            use super::*;

            pub(crate) fn write_all_at_should_write_whole_buffer_at_offset(file: &impl WriteFile) {
                file.write_all_at(b" world", 7).unwrap();
                file.write_all_at(b"C", 0).unwrap();

                assert_eq!(content(file), b"Content world");
            }
        }

        pub(crate) mod write_all_slices {
            use super::*;

            pub(crate) fn write_all_slices_should_write_all_buffers_at_position(
                file: &mut impl WriteFile,
            ) {
                file.seek(SeekFrom::Start(7)).unwrap();

                file.write_all_slices(&mut [
                    IoSlice::new(b" "),
                    IoSlice::new(b""),
                    IoSlice::new(b"wor"),
                    IoSlice::new(b"ld"),
                ])
                .unwrap();

                assert_eq!(content(file), b"content world");
                assert_eq!(file.stream_position().unwrap(), 13);
            }

            /// Takes a `file` opened for [`append`](OpenWriteOptions::append).
            pub(crate) fn write_all_slices_append_should_write_at_end(file: &mut impl WriteFile) {
                file.seek(SeekFrom::Start(0)).unwrap();

                file.write_all_slices(&mut [IoSlice::new(b" "), IoSlice::new(b"log")])
                    .unwrap();

                assert_eq!(content(file), b"content log");
            }
        }

        pub(crate) mod set_len {
            use super::*;

            pub(crate) fn set_len_should_truncate_or_extend_with_zeros(file: &impl WriteFile) {
                file.set_len(3).unwrap();
                assert_eq!(content(file), b"con");

                file.set_len(5).unwrap();
                assert_eq!(content(file), b"con\0\0");
            }

            pub(crate) fn set_len_should_keep_position(file: &mut impl WriteFile) {
                file.seek(SeekFrom::Start(5)).unwrap();

                file.set_len(2).unwrap();

                assert_eq!(file.stream_position().unwrap(), 5);
                file.write_all(b"!").unwrap();
                assert_eq!(content(file), b"co\0\0\0!");
            }

            pub(crate) fn set_len_invalid_length_should_fail(file: &impl WriteFile) {
                let error = file.set_len(u64::MAX).unwrap_err();

                assert_eq!(io_kind(&error), Some(std::io::ErrorKind::InvalidInput));
                assert!(
                    error
                        .description()
                        .ends_with(&file.path().display().to_string()),
                    "{error}"
                );
                assert_eq!(content(file), b"content");
            }
        }

        pub(crate) mod sync_all {
            use super::*;

            pub(crate) fn sync_all_should_succeed(file: &impl WriteFile) {
                file.write_all_at(b"C", 0).unwrap();

                file.sync_all().unwrap();

                assert_eq!(content(file), b"Content");
            }
        }

        pub(crate) mod sync_data {
            use super::*;

            pub(crate) fn sync_data_should_succeed(file: &impl WriteFile) {
                file.write_all_at(b"C", 0).unwrap();

                file.sync_data().unwrap();

                assert_eq!(content(file), b"Content");
            }
        }

        pub(crate) mod lock {
            use super::*;

            pub(crate) fn lock_should_lock_until_unlocked(file: &impl WriteFile) {
                file.lock().unwrap();
                file.unlock().unwrap();

                assert!(file.try_lock().unwrap());
            }
        }

        pub(crate) mod try_lock {
            use super::*;

            pub(crate) fn try_lock_should_lock_unlocked_file(file: &impl WriteFile) {
                assert!(file.try_lock().unwrap());
            }
        }
    }

    mod open_write_options {
        use super::*;

        #[test]
        fn open_write_options_should_set_each_flag() {
            let options = OpenWriteOptions::new()
                .append()
                .create()
                .create_new()
                .write_only()
                .truncate();

            assert!(options.append);
            assert!(options.create);
            assert!(options.create_new);
            assert!(options.write_only);
            assert!(options.truncate);
            assert_eq!(OpenWriteOptions::new(), OpenWriteOptions::default());
        }

        #[cfg(unix)]
        mod mode {
            use super::*;

            #[test]
            fn mode_should_set_mode_of_new_file() {
                assert_eq!(OpenWriteOptions::new().mode(0o600).mode, Some(0o600));
            }
        }
    }

    mod file_type {
        use super::*;

        #[test]
        fn file_type_should_tell_kind() {
            for (file_type, expected) in [
                (FileType::File, (true, false, false)),
                (FileType::Dir, (false, true, false)),
                (FileType::Symlink, (false, false, true)),
                (FileType::Other, (false, false, false)),
            ] {
                assert_eq!(
                    (
                        file_type.is_file(),
                        file_type.is_dir(),
                        file_type.is_symlink()
                    ),
                    expected,
                    "{file_type:?}"
                );
            }
        }

        #[cfg(unix)]
        mod from {
            use super::*;

            #[test]
            fn from_should_map_std_file_type() {
                let (test_dir, _fs) = test_fs();
                let dir = test_dir.path();
                std::fs::write(dir.join("file"), "").unwrap();
                std::os::unix::fs::symlink("file", dir.join("link")).unwrap();

                for (path, expected) in [
                    (dir.join("file"), FileType::File),
                    (dir.to_path_buf(), FileType::Dir),
                    (dir.join("link"), FileType::Symlink),
                    (PathBuf::from("/dev/null"), FileType::Other),
                ] {
                    let file_type = std::fs::symlink_metadata(&path).unwrap().file_type();

                    assert_eq!(FileType::from(file_type), expected, "{path:?}");
                }
            }
        }
    }

    mod dir_entry {
        use super::*;

        #[test]
        fn dir_entry_should_return_path_name_and_type() {
            let entry = DirEntry::new(PathBuf::from("/dir/a.txt"), FileType::File);

            assert_eq!(entry.path(), Path::new("/dir/a.txt"));
            assert_eq!(entry.file_name(), "a.txt");
            assert_eq!(entry.file_type(), FileType::File);
            assert_eq!(entry.clone(), entry);
        }

        mod file_name {
            use super::*;

            #[test]
            fn file_name_without_name_should_be_empty() {
                let entry = DirEntry::new(PathBuf::from("/"), FileType::Dir);

                assert_eq!(entry.file_name(), "");
            }
        }
    }

    #[cfg(unix)]
    mod unix_attributes {
        use super::*;

        mod attributes {
            use super::*;

            #[test]
            fn attributes_should_derive_flags_from_mode() {
                for (mode, readonly, executable) in [
                    (0o644, false, false),
                    (0o444, true, false),
                    (0o200, false, false),
                    (0o755, false, true),
                    (0o001, true, true),
                ] {
                    let attributes = UnixAttributes {
                        mode,
                        uid: 0,
                        gid: 0,
                    }
                    .attributes();

                    assert_eq!(
                        (
                            attributes.readonly,
                            attributes.executable,
                            attributes.hidden
                        ),
                        (readonly, executable, false),
                        "mode: {mode:o}"
                    );
                }
            }
        }
    }

    #[cfg(unix)]
    mod readonly_mode {
        use super::*;

        #[test]
        fn readonly_mode_should_clear_or_restore_owner_write() {
            for (mode, readonly, expected) in [
                (0o644, true, 0o444),
                (0o777, true, 0o555),
                (0o444, false, 0o644),
                (0o464, false, 0o464),
                (0o4755, true, 0o4555),
            ] {
                assert_eq!(
                    readonly_mode(mode, readonly),
                    expected,
                    "mode: {mode:o}, readonly: {readonly}"
                );
            }
        }
    }

    #[cfg(unix)]
    mod executable_mode {
        use super::*;

        #[test]
        fn executable_mode_should_set_execute_for_readers_or_clear_all() {
            for (mode, executable, expected) in [
                (0o644, true, 0o755),
                (0o600, true, 0o700),
                (0o000, true, 0o100),
                (0o744, true, 0o744),
                (0o755, false, 0o644),
            ] {
                assert_eq!(
                    executable_mode(mode, executable),
                    expected,
                    "mode: {mode:o}, executable: {executable}"
                );
            }
        }
    }

    mod metadata {
        use super::*;

        mod from_std {
            use super::*;

            #[test]
            fn from_std_should_convert_std_metadata() {
                let (test_dir, _fs) = test_fs();
                let path = test_dir.path().join(".hidden.sh");
                std::fs::write(&path, "hello").unwrap();
                #[cfg(unix)]
                set_mode(&path, 0o755);
                let std_metadata = std::fs::metadata(&path).unwrap();

                let metadata = Metadata::from_std(&path, &std_metadata);

                assert_eq!(metadata.file_type, FileType::File);
                assert_eq!(metadata.len, 5);
                assert_eq!(metadata.modified, std_metadata.modified().unwrap());
                assert!(!metadata.readonly);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;

                    assert!(metadata.executable);
                    assert!(metadata.hidden);
                    assert_eq!(metadata.mode, 0o755);
                    assert_eq!(metadata.uid, std_metadata.uid());
                    assert_eq!(metadata.gid, std_metadata.gid());
                }
            }
        }
    }

    mod platform_attributes {
        use super::*;

        #[cfg(unix)]
        #[test]
        fn platform_attributes_should_derive_hidden_and_restrict_executable_to_files() {
            let stored = Attributes {
                readonly: true,
                executable: true,
                hidden: false,
            };

            for (path, file_type, executable, hidden) in [
                ("/a.sh", FileType::File, true, false),
                ("/.a", FileType::File, true, true),
                ("/dir", FileType::Dir, false, false),
                ("/", FileType::Dir, false, false),
            ] {
                let attributes = platform_attributes(Path::new(path), file_type, stored);

                assert_eq!(
                    (
                        attributes.readonly,
                        attributes.executable,
                        attributes.hidden
                    ),
                    (true, executable, hidden),
                    "{path}"
                );
            }
        }

        #[cfg(windows)]
        #[test]
        fn platform_attributes_should_derive_executable_from_extension() {
            let stored = Attributes {
                readonly: false,
                executable: false,
                hidden: true,
            };

            for (path, file_type, executable) in [
                ("C:\\a.EXE", FileType::File, true),
                ("C:\\a.cmd", FileType::File, true),
                ("C:\\a.txt", FileType::File, false),
                ("C:\\a.exe", FileType::Dir, false),
            ] {
                let attributes = platform_attributes(Path::new(path), file_type, stored);

                assert_eq!(
                    (attributes.executable, attributes.hidden),
                    (executable, true),
                    "{path}"
                );
            }
        }
    }

    mod file_read_handle {
        use super::*;

        mod read {
            use super::*;

            mod read_vectored {
                use super::*;

                #[test]
                fn read_vectored_should_fill_buffers_in_order() {
                    let (_test_dir, fs) = test_fs();
                    fs.write("a.txt", "content").unwrap();
                    let mut file = fs.open_read("a.txt").unwrap();
                    let (mut low, mut high) = ([0; 3], [0; 4]);

                    let read = file
                        .read_vectored(&mut [IoSliceMut::new(&mut low), IoSliceMut::new(&mut high)])
                        .unwrap();

                    assert!(read > 0);
                    assert_eq!(
                        &[low.as_slice(), &high].concat()[..read],
                        &b"content"[..read]
                    );
                }
            }

            #[allow(clippy::module_inception)]
            mod read {
                use super::*;

                #[test]
                fn read_should_read_from_position() {
                    let (_test_dir, fs) = test_fs();
                    fs.write("a.txt", "content").unwrap();
                    let mut file = fs.open_read("a.txt").unwrap();
                    let mut buf = [0; 4];

                    assert_eq!(file.read(&mut buf).unwrap(), 4);
                    assert_eq!(file.read(&mut buf).unwrap(), 3);
                    assert_eq!(&buf[..3], b"ent");
                }
            }

            mod read_to_end {
                use super::*;

                #[test]
                fn read_to_end_should_append_rest_of_file() {
                    let (_test_dir, fs) = test_fs();
                    fs.write("a.txt", "content").unwrap();
                    let mut buf = b"old ".to_vec();

                    let read = fs.open_read("a.txt").unwrap().read_to_end(&mut buf);

                    assert_eq!(read.unwrap(), 7);
                    assert_eq!(buf, b"old content");
                }
            }

            mod read_to_string {
                use super::*;

                #[test]
                fn read_to_string_should_append_rest_of_file() {
                    let (_test_dir, fs) = test_fs();
                    fs.write("a.txt", "content").unwrap();
                    let mut buf = "old ".to_string();

                    let read = fs.open_read("a.txt").unwrap().read_to_string(&mut buf);

                    assert_eq!(read.unwrap(), 7);
                    assert_eq!(buf, "old content");
                }
            }
        }

        mod seek {
            use super::*;

            #[test]
            fn seek_should_move_position() {
                let (_test_dir, fs) = test_fs();
                fs.write("a.txt", "content").unwrap();
                let mut file = fs.open_read("a.txt").unwrap();
                let mut buf = String::new();

                assert_eq!(file.seek(SeekFrom::End(-4)).unwrap(), 3);
                file.read_to_string(&mut buf).unwrap();
                assert_eq!(buf, "tent");
            }
        }

        mod read_file {
            use super::*;

            /// Returns a test directory with the file `a.txt` holding
            /// `content` and a handle reading it.
            #[track_caller]
            fn content_file() -> (TestDir, FsImpl, FileReadHandle) {
                let (test_dir, fs) = test_fs();
                fs.write("a.txt", "content").unwrap();
                let file = fs.open_read("a.txt").unwrap();
                (test_dir, fs, file)
            }

            mod path {
                use super::*;

                #[test]
                fn path_should_return_path_resolved_against_current_dir() {
                    let (test_dir, fs, _file) = content_file();
                    fs.create_symlink_file("link", "a.txt").unwrap();

                    crate::fs::tests::read_file::path::path_should_return_path_resolved_against_current_dir(
                        &fs.open_read("link").unwrap(),
                        &test_dir.path().join("link"),
                    );
                }
            }

            mod metadata {
                use super::*;

                #[test]
                fn metadata_should_return_metadata_of_open_file() {
                    let (_test_dir, fs, file) = content_file();
                    crate::fs::tests::read_file::metadata::metadata_should_return_metadata_of_open_file(
                        &file,
                        &fs.metadata("a.txt").unwrap(),
                    );
                }
            }

            mod read_at {
                use super::*;

                #[test]
                fn read_at_should_read_at_offset_from_shared_handle() {
                    let (_test_dir, _fs, file) = content_file();
                    crate::fs::tests::read_file::read_at::read_at_should_read_at_offset_from_shared_handle(&file);
                }
            }

            mod read_exact_at {
                use super::*;

                #[test]
                fn read_exact_at_should_fill_buffer_or_fail_at_end() {
                    let (_test_dir, _fs, file) = content_file();
                    crate::fs::tests::read_file::read_exact_at::read_exact_at_should_fill_buffer_or_fail_at_end(&file);
                }
            }

            mod lock_shared {
                use super::*;

                #[test]
                fn lock_shared_should_lock_until_unlocked() {
                    let (_test_dir, _fs, file) = content_file();
                    crate::fs::tests::read_file::lock_shared::lock_shared_should_lock_until_unlocked(&file);
                }

                #[test]
                fn lock_shared_should_allow_only_other_shared_locks() {
                    let (_test_dir, fs) = test_fs();
                    fs.write("a.txt", "").unwrap();
                    let file = fs.open_read("a.txt").unwrap();

                    file.lock_shared().unwrap();

                    assert!(fs.open_read("a.txt").unwrap().try_lock_shared().unwrap());
                    assert!(!fs.open_write("a.txt").unwrap().try_lock().unwrap());
                }
            }

            mod try_lock_shared {
                use super::*;

                #[test]
                fn try_lock_shared_should_lock_unlocked_file() {
                    let (_test_dir, _fs, file) = content_file();
                    crate::fs::tests::read_file::try_lock_shared::try_lock_shared_should_lock_unlocked_file(&file);
                }

                #[test]
                fn try_lock_shared_should_fail_while_exclusively_locked() {
                    let (_test_dir, fs) = test_fs();
                    fs.write("a.txt", "").unwrap();
                    let file = fs.open_write("a.txt").unwrap();
                    file.lock().unwrap();
                    let other = fs.open_read("a.txt").unwrap();

                    assert!(!other.try_lock_shared().unwrap());
                    file.unlock().unwrap();
                    assert!(other.try_lock_shared().unwrap());
                }
            }

            mod unlock {
                use super::*;

                #[test]
                fn unlock_should_release_lock() {
                    let (_test_dir, _fs, file) = content_file();
                    crate::fs::tests::read_file::unlock::unlock_should_release_lock(&file);
                }

                #[test]
                fn unlock_should_let_other_handle_lock_exclusively() {
                    let (_test_dir, fs) = test_fs();
                    fs.write("a.txt", "").unwrap();
                    let file = fs.open_read("a.txt").unwrap();
                    file.lock_shared().unwrap();
                    let other = fs.open_write("a.txt").unwrap();

                    file.unlock().unwrap();

                    assert!(other.try_lock().unwrap());
                }
            }
        }
    }

    mod file_write_handle {
        use super::*;

        mod read {
            use super::*;

            mod read_vectored {
                use super::*;

                #[test]
                fn read_vectored_should_fill_buffers_in_order() {
                    let (_test_dir, fs) = test_fs();
                    fs.write("a.txt", "content").unwrap();
                    let mut file = fs.open_write("a.txt").unwrap();
                    let (mut low, mut high) = ([0; 3], [0; 4]);

                    let read = file
                        .read_vectored(&mut [IoSliceMut::new(&mut low), IoSliceMut::new(&mut high)])
                        .unwrap();

                    assert!(read > 0);
                    assert_eq!(
                        &[low.as_slice(), &high].concat()[..read],
                        &b"content"[..read]
                    );
                }
            }

            #[allow(clippy::module_inception)]
            mod read {
                use super::*;

                #[test]
                fn read_should_read_what_was_written() {
                    let (_test_dir, fs) = test_fs();
                    let mut file = fs.open_write("a.txt").unwrap();
                    file.write_all(b"content").unwrap();
                    file.seek(SeekFrom::Start(0)).unwrap();
                    let mut buf = [0; 3];

                    assert_eq!(file.read(&mut buf).unwrap(), 3);
                    assert_eq!(&buf, b"con");
                }
            }

            mod read_to_end {
                use super::*;

                #[test]
                fn read_to_end_should_append_rest_of_file() {
                    let (_test_dir, fs) = test_fs();
                    fs.write("a.txt", "content").unwrap();
                    let mut buf = Vec::new();

                    let read = fs.open_write("a.txt").unwrap().read_to_end(&mut buf);

                    assert_eq!(read.unwrap(), 7);
                    assert_eq!(buf, b"content");
                }
            }

            mod read_to_string {
                use super::*;

                #[test]
                fn read_to_string_should_append_rest_of_file() {
                    let (_test_dir, fs) = test_fs();
                    fs.write("a.txt", "content").unwrap();
                    let mut buf = String::new();

                    let read = fs.open_write("a.txt").unwrap().read_to_string(&mut buf);

                    assert_eq!(read.unwrap(), 7);
                    assert_eq!(buf, "content");
                }
            }
        }

        mod seek {
            use super::*;

            #[test]
            fn seek_should_move_position() {
                let (_test_dir, fs) = test_fs();
                fs.write("a.txt", "content").unwrap();
                let mut file = fs.open_write("a.txt").unwrap();

                assert_eq!(file.seek(SeekFrom::Current(3)).unwrap(), 3);
            }
        }

        mod write {
            use super::*;

            mod write_vectored {
                use super::*;

                #[test]
                fn write_vectored_should_write_buffers_in_order() {
                    let (_test_dir, fs) = test_fs();
                    let mut file = fs.open_write("a.txt").unwrap();

                    let written = file
                        .write_vectored(&[IoSlice::new(b"con"), IoSlice::new(b"tent")])
                        .unwrap();

                    assert!(written > 0);
                    drop(file);
                    assert_eq!(fs.read_to_string("a.txt").unwrap(), &"content"[..written]);
                }
            }

            #[allow(clippy::module_inception)]
            mod write {
                use super::*;

                #[test]
                fn write_should_write_at_position() {
                    let (_test_dir, fs) = test_fs();
                    fs.write("a.txt", "content").unwrap();
                    let mut file = fs.open_write("a.txt").unwrap();
                    file.seek(SeekFrom::Start(3)).unwrap();

                    assert_eq!(file.write(b"TENT").unwrap(), 4);
                    assert_eq!(fs.read_to_string("a.txt").unwrap(), "conTENT");
                }
            }

            mod flush {
                use super::*;

                #[test]
                fn flush_should_succeed() {
                    let (_test_dir, fs) = test_fs();
                    let mut file = fs.open_write("a.txt").unwrap();

                    file.flush().unwrap();
                }
            }
        }

        mod read_file {
            use super::*;

            /// Returns a test directory with the file `a.txt` holding
            /// `content` and a handle reading and writing it.
            #[track_caller]
            fn content_file() -> (TestDir, FsImpl, FileWriteHandle) {
                let (test_dir, fs) = test_fs();
                fs.write("a.txt", "content").unwrap();
                let file = fs.open_write("a.txt").unwrap();
                (test_dir, fs, file)
            }

            mod path {
                use super::*;

                #[test]
                fn path_should_return_path_resolved_against_current_dir() {
                    let (test_dir, fs, _file) = content_file();
                    fs.create_symlink_file("link", "b.txt").unwrap();

                    crate::fs::tests::read_file::path::path_should_return_path_resolved_against_current_dir(
                        &fs.open_write("link").unwrap(),
                        &test_dir.path().join("link"),
                    );
                }
            }

            mod metadata {
                use super::*;

                #[test]
                fn metadata_should_return_metadata_of_open_file() {
                    let (_test_dir, fs, file) = content_file();
                    crate::fs::tests::read_file::metadata::metadata_should_return_metadata_of_open_file(
                        &file,
                        &fs.metadata("a.txt").unwrap(),
                    );
                }
            }

            mod read_at {
                use super::*;

                #[test]
                fn read_at_should_read_at_offset_from_shared_handle() {
                    let (_test_dir, _fs, file) = content_file();
                    crate::fs::tests::read_file::read_at::read_at_should_read_at_offset_from_shared_handle(&file);
                }
            }

            mod read_exact_at {
                use super::*;

                #[test]
                fn read_exact_at_should_fill_buffer_or_fail_at_end() {
                    let (_test_dir, _fs, file) = content_file();
                    crate::fs::tests::read_file::read_exact_at::read_exact_at_should_fill_buffer_or_fail_at_end(&file);
                }
            }

            mod lock_shared {
                use super::*;

                #[test]
                fn lock_shared_should_lock_until_unlocked() {
                    let (_test_dir, _fs, file) = content_file();
                    crate::fs::tests::read_file::lock_shared::lock_shared_should_lock_until_unlocked(&file);
                }

                #[test]
                fn lock_shared_should_exclude_exclusive_lock_of_other_handle() {
                    let (_test_dir, fs) = test_fs();
                    let file = fs.open_write("a.txt").unwrap();

                    file.lock_shared().unwrap();
                    assert!(!fs.open_write("a.txt").unwrap().try_lock().unwrap());
                    file.unlock().unwrap();

                    assert!(fs.open_write("a.txt").unwrap().try_lock().unwrap());
                }
            }

            mod try_lock_shared {
                use super::*;

                #[test]
                fn try_lock_shared_should_lock_unlocked_file() {
                    let (_test_dir, _fs, file) = content_file();
                    crate::fs::tests::read_file::try_lock_shared::try_lock_shared_should_lock_unlocked_file(&file);
                }
            }

            mod unlock {
                use super::*;

                #[test]
                fn unlock_should_release_lock() {
                    let (_test_dir, _fs, file) = content_file();
                    crate::fs::tests::read_file::unlock::unlock_should_release_lock(&file);
                }
            }
        }

        mod write_file {
            use super::*;

            /// Returns a test directory with the file `a.txt` holding
            /// `content` and a handle opened on it with `options`.
            #[track_caller]
            fn content_file(options: OpenWriteOptions) -> (TestDir, FsImpl, FileWriteHandle) {
                let (test_dir, fs) = test_fs();
                fs.write("a.txt", "content").unwrap();
                let file = fs.open_write_options("a.txt", options).unwrap();
                (test_dir, fs, file)
            }

            mod write_at {
                use super::*;

                #[test]
                fn write_at_should_write_at_offset_and_extend_file() {
                    let (_test_dir, _fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::write_at::write_at_should_write_at_offset_and_extend_file(&file);
                }

                #[cfg(target_os = "linux")]
                #[test]
                fn write_at_append_should_ignore_offset() {
                    let (_test_dir, _fs, file) = content_file(OpenWriteOptions::new().append());
                    crate::fs::tests::write_file::write_at::write_at_append_should_ignore_offset(
                        &file,
                    );
                }
            }

            mod write_all_at {
                use super::*;

                #[test]
                fn write_all_at_should_write_whole_buffer_at_offset() {
                    let (_test_dir, _fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::write_all_at::write_all_at_should_write_whole_buffer_at_offset(&file);
                }
            }

            mod write_all_slices {
                use super::*;

                #[test]
                fn write_all_slices_should_write_all_buffers_at_position() {
                    let (_test_dir, _fs, mut file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::write_all_slices::write_all_slices_should_write_all_buffers_at_position(&mut file);
                }

                #[test]
                fn write_all_slices_append_should_write_at_end() {
                    let (_test_dir, _fs, mut file) = content_file(OpenWriteOptions::new().append());
                    crate::fs::tests::write_file::write_all_slices::write_all_slices_append_should_write_at_end(&mut file);
                }
            }

            mod set_len {
                use super::*;

                #[test]
                fn set_len_should_truncate_or_extend_with_zeros() {
                    let (_test_dir, _fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::set_len::set_len_should_truncate_or_extend_with_zeros(&file);
                }

                #[test]
                fn set_len_should_keep_position() {
                    let (_test_dir, _fs, mut file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::set_len::set_len_should_keep_position(&mut file);
                }

                #[test]
                fn set_len_invalid_length_should_fail() {
                    let (_test_dir, _fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::set_len::set_len_invalid_length_should_fail(
                        &file,
                    );
                }
            }

            mod sync_all {
                use super::*;

                #[test]
                fn sync_all_should_succeed() {
                    let (_test_dir, _fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::sync_all::sync_all_should_succeed(&file);
                }
            }

            mod sync_data {
                use super::*;

                #[test]
                fn sync_data_should_succeed() {
                    let (_test_dir, _fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::sync_data::sync_data_should_succeed(&file);
                }
            }

            mod lock {
                use super::*;

                #[test]
                fn lock_should_lock_until_unlocked() {
                    let (_test_dir, _fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::lock::lock_should_lock_until_unlocked(&file);
                }

                #[test]
                fn lock_should_exclude_other_locks() {
                    let (_test_dir, fs) = test_fs();
                    let file = fs.open_write("a.txt").unwrap();

                    file.lock().unwrap();

                    assert!(!fs.open_write("a.txt").unwrap().try_lock().unwrap());
                    assert!(!fs.open_read("a.txt").unwrap().try_lock_shared().unwrap());
                }
            }

            mod try_lock {
                use super::*;

                #[test]
                fn try_lock_should_lock_unlocked_file() {
                    let (_test_dir, _fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::try_lock::try_lock_should_lock_unlocked_file(
                        &file,
                    );
                }

                #[test]
                fn try_lock_should_fail_while_other_handle_holds_lock() {
                    let (_test_dir, fs) = test_fs();
                    let file = fs.open_write("a.txt").unwrap();

                    assert!(file.try_lock().unwrap());
                    assert!(!fs.open_write("a.txt").unwrap().try_lock().unwrap());
                }
            }
        }
    }

    #[cfg(unix)]
    mod read_dir {
        use super::*;

        mod iterator {
            use super::*;

            #[test]
            fn iterator_should_return_every_entry_with_its_type() {
                // GIVEN a directory with each type of entry, one named with
                // invalid UTF-8
                let (test_dir, fs) = test_fs();
                let dir = test_dir.path();
                std::fs::create_dir(dir.join("dir")).unwrap();
                std::fs::write(dir.join("file"), "").unwrap();
                std::os::unix::fs::symlink("missing", dir.join("link")).unwrap();
                let _socket = std::os::unix::net::UnixListener::bind(dir.join("socket")).unwrap();
                let invalid = <OsStr as std::os::unix::ffi::OsStrExt>::from_bytes(b"\xff");
                std::fs::write(dir.join(invalid), "").unwrap();

                // WHEN it is read
                let mut entries: Vec<DirEntry> =
                    fs.read_dir(".").unwrap().map(Result::unwrap).collect();
                entries.sort_by(|a, b| a.path().cmp(b.path()));

                // THEN every entry is returned under the path as given
                assert_eq!(
                    entries,
                    [
                        DirEntry::new(dir.join("./dir"), FileType::Dir),
                        DirEntry::new(dir.join("./file"), FileType::File),
                        DirEntry::new(dir.join("./link"), FileType::Symlink),
                        DirEntry::new(dir.join("./socket"), FileType::Other),
                        DirEntry::new(dir.join(".").join(invalid), FileType::File),
                    ]
                );
            }
        }
    }

    mod fs_impl {
        use super::*;

        #[test]
        fn fs_impl_empty_path_should_not_exist() {
            let (_test_dir, fs) = test_fs();
            fs.write("a", "").unwrap();
            let results = [
                fs.open_read("").map(|_| ()),
                fs.open_write("").map(|_| ()),
                fs.read("").map(|_| ()),
                fs.read_to_string("").map(|_| ()),
                fs.write("", ""),
                fs.read_dir("").map(|_| ()),
                fs.create_dir(""),
                fs.create_dir_all(""),
                fs.exists("").map(|_| ()),
                fs.metadata("").map(|_| ()),
                fs.symlink_metadata("").map(|_| ()),
                fs.create_symlink_file("", "a"),
                fs.create_symlink_dir("", "a"),
                fs.read_link("").map(|_| ()),
                fs.canonicalize("").map(|_| ()),
                fs.create_hard_link("", "a"),
                fs.create_hard_link("b", ""),
                fs.copy("", "b").map(|_| ()),
                fs.copy("a", "").map(|_| ()),
                fs.copy_dir_all("", "b"),
                fs.copy_dir_all(".", ""),
                fs.remove_file(""),
                fs.remove_dir(""),
                fs.remove_dir_all(""),
                fs.rename("", "b"),
                fs.rename("a", ""),
                fs.set_readonly("", true),
                #[cfg(unix)]
                fs.set_executable("", true),
                fs.set_modified("", SystemTime::UNIX_EPOCH),
                #[cfg(unix)]
                fs.set_mode("", 0o644),
            ];

            for (index, result) in results.iter().enumerate() {
                let error = result.as_ref().expect_err(&index.to_string());

                assert_eq!(
                    io_kind(error),
                    Some(std::io::ErrorKind::NotFound),
                    "{index}: {error}"
                );
            }
        }

        mod with_env {
            use super::*;

            #[test]
            fn with_env_should_resolve_against_changed_current_dir() {
                // GIVEN an `FsImpl` sharing an env with a test directory
                let test_dir = TestDir::new();
                let env = EnvImpl::new(Vec::new(), PathBuf::from("/"));
                let fs = FsImpl::with_env(env.clone());
                std::fs::write(test_dir.path().join("a.txt"), "content").unwrap();

                // WHEN the env changes its current directory
                env.replace_current_dir(test_dir.path().to_path_buf());

                // THEN relative paths resolve against it
                assert_eq!(fs.read_to_string("a.txt").unwrap(), "content");
            }
        }

        mod fs {
            use super::*;

            mod open_read {
                use super::*;

                #[test]
                fn open_read_should_open_relative_and_absolute_path() {
                    let (test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_read::open_read_should_open_relative_and_absolute_path(
                        &fs,
                        test_dir.path(),
                    );
                }

                #[test]
                fn open_read_missing_file_should_fail_with_path() {
                    let (test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_read::open_read_missing_file_should_fail_with_path(
                        &fs,
                        test_dir.path(),
                    );
                }
            }

            mod open_write {
                use super::*;

                #[test]
                fn open_write_should_create_file_without_truncating() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_write::open_write_should_create_file_without_truncating(
                        &fs,
                    );
                }

                #[test]
                fn open_write_should_open_for_reading() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_write::open_write_should_open_for_reading(&fs);
                }
            }

            mod open_write_options {
                use super::*;

                #[test]
                fn open_write_options_should_append_or_truncate() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_write_options::open_write_options_should_append_or_truncate(
                        &fs,
                    );
                }

                #[test]
                fn open_write_options_append_should_write_at_end_regardless_of_position() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_write_options::open_write_options_append_should_write_at_end_regardless_of_position(
                        &fs,
                    );
                }

                #[test]
                fn open_write_options_write_only_should_not_be_readable() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_write_options::open_write_options_write_only_should_not_be_readable(
                        &fs,
                    );
                }

                #[test]
                fn open_write_options_without_create_missing_file_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_write_options::open_write_options_without_create_missing_file_should_fail(
                        &fs,
                    );
                }

                #[test]
                fn open_write_options_create_new_existing_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_write_options::open_write_options_create_new_existing_should_fail(
                        &fs,
                    );
                }

                #[test]
                fn open_write_options_append_and_truncate_should_require_create_new() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_write_options::open_write_options_append_and_truncate_should_require_create_new(
                        &fs,
                    );
                }

                #[test]
                fn open_write_options_create_should_follow_dangling_symlink() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_write_options::open_write_options_create_should_follow_dangling_symlink(
                        &fs,
                    );
                }

                #[test]
                fn open_write_options_create_with_separator_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_write_options::open_write_options_create_with_separator_should_fail(
                        &fs,
                    );
                }

                #[cfg(unix)]
                #[test]
                fn open_write_options_mode_should_set_mode_of_new_file() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_write_options::open_write_options_mode_should_set_mode_of_new_file(
                        &fs,
                    );
                }

                #[test]
                fn open_write_options_invalid_target_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::open_write_options::open_write_options_invalid_target_should_fail(
                        &fs,
                    );
                }
            }

            mod read {
                use super::*;

                #[test]
                fn read_should_return_content() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::read::read_should_return_content(&fs);
                }

                #[test]
                fn read_missing_file_should_fail_with_path() {
                    let (test_dir, fs) = test_fs();
                    crate::fs::tests::fs::read::read_missing_file_should_fail_with_path(
                        &fs,
                        test_dir.path(),
                    );
                }

                #[test]
                fn read_directory_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::read::read_directory_should_fail(&fs);
                }
            }

            mod read_to_string {
                use super::*;

                #[test]
                fn read_to_string_should_return_content() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::read_to_string::read_to_string_should_return_content(&fs);
                }

                #[test]
                fn read_to_string_invalid_utf8_should_fail_as_invalid_data() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::read_to_string::read_to_string_invalid_utf8_should_fail_as_invalid_data(
                        &fs,
                    );
                }
            }

            mod write {
                use super::*;

                #[test]
                fn write_should_replace_content() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::write::write_should_replace_content(&fs);
                }

                #[cfg(unix)]
                #[test]
                fn write_should_write_file_that_cannot_be_read() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::write::write_should_write_file_that_cannot_be_read(&fs);
                }

                #[test]
                fn write_missing_parent_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::write::write_missing_parent_should_fail(&fs);
                }

                #[test]
                fn write_directory_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::write::write_directory_should_fail(&fs);
                }

                #[test]
                fn write_should_update_modified_time() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::write::write_should_update_modified_time(&fs);
                }

                #[test]
                fn write_new_file_should_update_modified_time_of_parent() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::write::write_new_file_should_update_modified_time_of_parent(
                        &fs,
                    );
                }

                #[test]
                fn write_path_with_nul_byte_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::write::write_path_with_nul_byte_should_fail(&fs);
                }

                #[cfg(unix)]
                #[test]
                fn write_file_with_trailing_current_dir_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::write::write_file_with_trailing_current_dir_should_fail(
                        &fs,
                    );
                }
            }

            mod read_dir {
                use super::*;

                #[test]
                fn read_dir_should_list_entries_under_given_path() {
                    let (test_dir, fs) = test_fs();
                    crate::fs::tests::fs::read_dir::read_dir_should_list_entries_under_given_path(
                        &fs,
                        test_dir.path(),
                    );
                }

                #[test]
                fn read_dir_invalid_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::read_dir::read_dir_invalid_path_should_fail(&fs);
                }
            }

            mod create_dir {
                use super::*;

                #[test]
                fn create_dir_should_create_only_new_dir() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_dir::create_dir_should_create_only_new_dir(&fs);
                }

                #[test]
                fn create_dir_invalid_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_dir::create_dir_invalid_path_should_fail(&fs);
                }

                #[cfg(unix)]
                #[test]
                fn create_dir_with_trailing_current_dir_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_dir::create_dir_with_trailing_current_dir_should_fail(
                        &fs,
                    );
                }
            }

            mod create_dir_all {
                use super::*;

                #[test]
                fn create_dir_all_should_create_missing_parents() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_dir_all::create_dir_all_should_create_missing_parents(
                        &fs,
                    );
                }

                #[test]
                fn create_dir_all_existing_non_dir_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_dir_all::create_dir_all_existing_non_dir_should_fail(
                        &fs,
                    );
                }
            }

            mod exists {
                use super::*;

                #[test]
                fn exists_should_follow_symlinks() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::exists::exists_should_follow_symlinks(&fs);
                }

                #[cfg(unix)]
                #[test]
                fn exists_file_as_parent_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::exists::exists_file_as_parent_should_fail(&fs);
                }

                #[cfg(windows)]
                #[test]
                fn exists_file_as_parent_should_not_exist() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::exists::exists_file_as_parent_should_not_exist(&fs);
                }
            }

            mod metadata {
                use super::*;

                #[test]
                fn metadata_should_follow_symlinks() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::metadata::metadata_should_follow_symlinks(&fs);
                }

                #[test]
                fn metadata_missing_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::metadata::metadata_missing_path_should_fail(&fs);
                }

                #[cfg(unix)]
                #[test]
                fn metadata_file_with_trailing_current_dir_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::metadata::metadata_file_with_trailing_current_dir_should_fail(
                        &fs,
                    );
                }

                #[cfg(unix)]
                #[test]
                fn metadata_special_file_should_be_other() {
                    let (_test_dir, fs) = test_fs();

                    assert_eq!(fs.metadata("/dev/null").unwrap().file_type, FileType::Other);
                }
            }

            mod symlink_metadata {
                use super::*;

                #[test]
                fn symlink_metadata_should_not_follow_symlink() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::symlink_metadata::symlink_metadata_should_not_follow_symlink(
                        &fs,
                    );
                }
            }

            mod create_symlink_file {
                use super::*;

                #[test]
                fn create_symlink_file_should_resolve_relative_target_against_link_parent() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_symlink_file::create_symlink_file_should_resolve_relative_target_against_link_parent(
                        &fs,
                    );
                }

                #[test]
                fn create_symlink_file_with_parent_dir_in_target_should_resolve_per_os() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_symlink_file::create_symlink_file_with_parent_dir_in_target_should_resolve_per_os(
                        &fs,
                    );
                }

                #[test]
                fn create_symlink_file_current_dir_target_should_resolve() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_symlink_file::create_symlink_file_current_dir_target_should_resolve(
                        &fs,
                    );
                }

                #[test]
                fn create_symlink_file_should_allow_dangling_target() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_symlink_file::create_symlink_file_should_allow_dangling_target(
                        &fs,
                    );
                }

                #[test]
                fn create_symlink_file_existing_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_symlink_file::create_symlink_file_existing_path_should_fail(
                        &fs,
                    );
                }

                #[test]
                fn create_symlink_file_invalid_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_symlink_file::create_symlink_file_invalid_path_should_fail(
                        &fs,
                    );
                }
            }

            mod create_symlink_dir {
                use super::*;

                #[test]
                fn create_symlink_dir_should_link_dir_created_later() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_symlink_dir::create_symlink_dir_should_link_dir_created_later(
                        &fs,
                    );
                }
            }

            mod read_link {
                use super::*;

                #[test]
                fn read_link_should_return_target_as_stored() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::read_link::read_link_should_return_target_as_stored(&fs);
                }

                #[test]
                fn read_link_not_symlink_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::read_link::read_link_not_symlink_should_fail(&fs);
                }

                #[test]
                fn read_link_missing_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::read_link::read_link_missing_path_should_fail(&fs);
                }
            }

            mod canonicalize {
                use super::*;

                #[test]
                fn canonicalize_should_resolve_symlinks_and_parent_dirs() {
                    let (test_dir, fs) = test_fs();
                    crate::fs::tests::fs::canonicalize::canonicalize_should_resolve_symlinks_and_parent_dirs(
                        &fs,
                        &test_dir.path().canonicalize().unwrap(),
                    );
                }

                #[test]
                fn canonicalize_missing_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::canonicalize::canonicalize_missing_path_should_fail(&fs);
                }
            }

            mod create_hard_link {
                use super::*;

                #[test]
                fn create_hard_link_should_share_content() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_hard_link::create_hard_link_should_share_content(
                        &fs,
                    );
                }

                #[test]
                fn create_hard_link_to_symlink_should_link_symlink_itself() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_hard_link::create_hard_link_to_symlink_should_link_symlink_itself(
                        &fs,
                    );
                }

                #[test]
                fn create_hard_link_invalid_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::create_hard_link::create_hard_link_invalid_path_should_fail(
                        &fs,
                    );
                }

                #[test]
                fn create_hard_link_missing_target_should_fail_with_both_paths() {
                    let (test_dir, fs) = test_fs();

                    let error = fs.create_hard_link("b.txt", "a.txt").unwrap_err();

                    assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
                    assert!(error.description().ends_with(&format!(
                        ": {} -> {}",
                        test_dir.path().join("a.txt").display(),
                        test_dir.path().join("b.txt").display()
                    )));
                }
            }

            mod copy {
                use super::*;

                #[test]
                fn copy_should_overwrite_target() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::copy::copy_should_overwrite_target(&fs);
                }

                #[test]
                fn copy_should_copy_content_and_permissions_through_symlinks() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::copy::copy_should_copy_content_and_permissions_through_symlinks(
                        &fs,
                    );
                }

                #[test]
                fn copy_should_follow_dangling_symlink_target() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::copy::copy_should_follow_dangling_symlink_target(&fs);
                }

                #[test]
                fn copy_onto_itself_should_fail_keeping_content() {
                    // WHY: std::fs::copy truncates a file copied onto itself or onto
                    // a link of it.
                }

                #[test]
                fn copy_dir_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::copy::copy_dir_should_fail(&fs);
                }

                #[test]
                fn copy_missing_source_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::copy::copy_missing_source_should_fail(&fs);
                }

                #[test]
                fn copy_invalid_target_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::copy::copy_invalid_target_should_fail(&fs);
                }

                #[cfg(windows)]
                #[test]
                fn copy_hidden_should_copy_attribute_and_overwrite_only_from_hidden_source() {
                    // GIVEN: a hidden source
                    let (test_dir, fs) = test_fs();
                    fs.write("a.txt", "content").unwrap();
                    hide(&test_dir.path().join("a.txt"));

                    // WHEN: it is copied to a new target
                    fs.copy("a.txt", "b.txt").unwrap();

                    // THEN: the copy is hidden too
                    assert!(fs.metadata("b.txt").unwrap().hidden);
                    // AND: the hidden source overwrites the hidden target
                    fs.copy("a.txt", "b.txt").unwrap();
                    // AND: a source that is not hidden is denied
                    fs.write("c.txt", "new").unwrap();
                    let error = fs.copy("c.txt", "b.txt").unwrap_err();
                    assert_eq!(io_kind(&error), Some(std::io::ErrorKind::PermissionDenied));
                    assert_eq!(fs.read_to_string("b.txt").unwrap(), "content");
                }
            }

            mod copy_dir_all {
                use super::*;

                #[test]
                fn copy_dir_all_should_copy_tree_with_symlinks_as_symlinks() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::copy_dir_all::copy_dir_all_should_copy_tree_with_symlinks_as_symlinks(
                        &fs,
                    );
                }

                #[cfg(unix)]
                #[test]
                fn copy_dir_all_should_copy_dir_permissions() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::copy_dir_all::copy_dir_all_should_copy_dir_permissions(
                        &fs,
                    );
                }

                #[test]
                fn copy_dir_all_invalid_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::copy_dir_all::copy_dir_all_invalid_path_should_fail(&fs);
                }
            }

            mod remove_file {
                use super::*;

                #[test]
                fn remove_file_should_remove_file_or_symlink_only() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::remove_file::remove_file_should_remove_file_or_symlink_only(
                        &fs,
                    );
                }

                #[test]
                fn remove_file_invalid_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::remove_file::remove_file_invalid_path_should_fail(&fs);
                }

                #[cfg(unix)]
                #[test]
                fn remove_file_symlink_to_dir_with_trailing_separator_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::remove_file::remove_file_symlink_to_dir_with_trailing_separator_should_fail(
                        &fs,
                    );
                }
            }

            mod remove_dir {
                use super::*;

                #[test]
                fn remove_dir_should_remove_only_empty_dir() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::remove_dir::remove_dir_should_remove_only_empty_dir(&fs);
                }

                #[test]
                fn remove_dir_invalid_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::remove_dir::remove_dir_invalid_path_should_fail(&fs);
                }

                #[cfg(unix)]
                #[test]
                fn remove_dir_symlink_to_dir_with_trailing_separator_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::remove_dir::remove_dir_symlink_to_dir_with_trailing_separator_should_fail(
                        &fs,
                    );
                }
            }

            mod remove_dir_all {
                use super::*;

                #[test]
                fn remove_dir_all_should_remove_tree_but_not_symlink_targets() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::remove_dir_all::remove_dir_all_should_remove_tree_but_not_symlink_targets(
                        &fs,
                    );
                }

                #[test]
                fn remove_dir_all_root_should_fail_without_removing_anything() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::remove_dir_all::remove_dir_all_root_should_fail_without_removing_anything(
                        &fs,
                    );
                }

                #[test]
                fn remove_dir_all_trailing_separator_should_remove_dir() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::remove_dir_all::remove_dir_all_trailing_separator_should_remove_dir(
                        &fs,
                    );
                }

                #[test]
                fn remove_dir_all_symlink_with_trailing_separator_should_fail_without_removing_target()
                 {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::remove_dir_all::remove_dir_all_symlink_with_trailing_separator_should_fail_without_removing_target(
                        &fs,
                    );
                }

                #[test]
                fn remove_dir_all_missing_path_with_trailing_separator_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::remove_dir_all::remove_dir_all_missing_path_with_trailing_separator_should_fail(
                        &fs,
                    );
                }

                #[test]
                fn remove_dir_all_invalid_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::remove_dir_all::remove_dir_all_invalid_path_should_fail(
                        &fs,
                    );
                }
            }

            mod rename {
                use super::*;

                #[test]
                fn rename_should_replace_target_atomically() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::rename::rename_should_replace_target_atomically(&fs);
                }

                #[test]
                fn rename_should_move_tree() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::rename::rename_should_move_tree(&fs);
                }

                #[test]
                fn rename_to_new_path_should_move_entry() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::rename::rename_to_new_path_should_move_entry(&fs);
                }

                #[test]
                fn rename_same_entry_should_do_nothing() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::rename::rename_same_entry_should_do_nothing(&fs);
                }

                #[test]
                fn rename_hard_link_onto_same_file_should_do_nothing() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::rename::rename_hard_link_onto_same_file_should_do_nothing(
                        &fs,
                    );
                }

                #[test]
                fn rename_invalid_source_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::rename::rename_invalid_source_should_fail(&fs);
                }

                #[test]
                fn rename_invalid_target_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::rename::rename_invalid_target_should_fail(&fs);
                }

                #[test]
                fn rename_target_with_separator_should_require_dir() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::rename::rename_target_with_separator_should_require_dir(
                        &fs,
                    );
                }

                #[test]
                fn rename_missing_source_should_fail_with_both_paths() {
                    let (test_dir, fs) = test_fs();

                    let error = fs.rename("a.txt", "b.txt").unwrap_err();

                    assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
                    assert!(error.description().ends_with(&format!(
                        ": {} -> {}",
                        test_dir.path().join("a.txt").display(),
                        test_dir.path().join("b.txt").display()
                    )));
                }
            }

            mod set_readonly {
                use super::*;

                #[cfg(unix)]
                #[test]
                fn set_readonly_should_map_to_write_bits() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::set_readonly::set_readonly_should_map_to_write_bits(&fs);
                }

                #[cfg(windows)]
                #[test]
                fn set_readonly_should_set_and_clear_attribute() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::set_readonly::set_readonly_should_set_and_clear_attribute(
                        &fs,
                    );
                }

                #[test]
                fn set_readonly_missing_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::set_readonly::set_readonly_missing_path_should_fail(&fs);
                }
            }

            #[cfg(unix)]
            mod set_executable {
                use super::*;

                #[test]
                fn set_executable_should_map_to_execute_bits() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::set_executable::set_executable_should_map_to_execute_bits(
                        &fs,
                    );
                }

                #[test]
                fn set_executable_dir_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::set_executable::set_executable_dir_should_fail(&fs);
                }

                #[test]
                fn set_executable_missing_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::set_executable::set_executable_missing_path_should_fail(
                        &fs,
                    );
                }
            }

            mod set_modified {
                use super::*;

                #[test]
                fn set_modified_should_set_modification_time() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::set_modified::set_modified_should_set_modification_time(
                        &fs,
                    );
                }

                #[test]
                fn set_modified_missing_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::set_modified::set_modified_missing_path_should_fail(&fs);
                }
            }

            #[cfg(unix)]
            mod set_mode {
                use super::*;

                #[test]
                fn set_mode_should_set_permission_bits_only() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::set_mode::set_mode_should_set_permission_bits_only(&fs);
                }

                #[test]
                fn set_mode_missing_path_should_fail() {
                    let (_test_dir, fs) = test_fs();
                    crate::fs::tests::fs::set_mode::set_mode_missing_path_should_fail(&fs);
                }
            }
        }
    }

    mod copy_tree {
        use super::*;

        /// Returns a `TestFs` with the tree `/src` holding the directory
        /// `sub`, the file `sub/a.txt` and the symlink `link`.
        fn fs_with_tree() -> TestFs {
            let fs = TestFs::new();
            fs.create_dir_all("/src/sub").unwrap();
            fs.write("/src/sub/a.txt", "content").unwrap();
            fs.create_symlink_file("/src/link", "sub/a.txt").unwrap();
            fs
        }

        #[test]
        fn copy_tree_should_copy_dirs_files_symlinks_and_permissions() {
            let fs = fs_with_tree();
            fs.set_readonly("/src/sub", true).unwrap();

            copy_tree(&fs, Path::new("/src"), Path::new("/copy")).unwrap();

            assert_eq!(fs.read_to_string("/copy/sub/a.txt").unwrap(), "content");
            assert_eq!(fs.read_link("/copy/link").unwrap(), Path::new("sub/a.txt"));
            assert!(fs.metadata("/copy/sub").unwrap().readonly);
            assert!(!fs.metadata("/copy").unwrap().readonly);
        }

        #[test]
        fn copy_tree_should_fail_for_each_failing_step() {
            for (op, path, kind) in [
                (FsOp::Metadata, "/src", std::io::ErrorKind::NotFound),
                (FsOp::ReadDir, "/src/sub", std::io::ErrorKind::Other),
                (FsOp::CreateDir, "/copy", std::io::ErrorKind::AlreadyExists),
                (FsOp::Open, "/src/sub/a.txt", std::io::ErrorKind::Other),
                (FsOp::ReadLink, "/src/link", std::io::ErrorKind::Other),
                (FsOp::CreateLink, "/copy/link", std::io::ErrorKind::Other),
                (FsOp::Metadata, "/src/sub", std::io::ErrorKind::Other),
                (FsOp::SetPermissions, "/copy", std::io::ErrorKind::Other),
            ] {
                let fs = fs_with_tree();
                fs.set_io_error(op, path, kind);

                assert_eq!(
                    copy_tree(&fs, Path::new("/src"), Path::new("/copy"))
                        .map_err(|error| io_kind(&error)),
                    Err(Some(kind)),
                    "{op:?} {path}"
                );
            }
        }

        #[test]
        fn copy_tree_file_should_fail() {
            let fs = fs_with_tree();

            let error =
                copy_tree(&fs, Path::new("/src/sub/a.txt"), Path::new("/copy")).unwrap_err();

            assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotADirectory));
            assert!(!fs.exists("/copy").unwrap());
        }
    }

    mod add_entry {
        use super::*;

        #[test]
        fn add_entry_should_sort_entries_and_return_dirs() {
            let mut tree = Tree::default();
            let target = Path::new("/copy");

            for (entry, expected) in [
                (
                    DirEntry::new(PathBuf::from("/src/dir"), FileType::Dir),
                    Some((PathBuf::from("/src/dir"), PathBuf::from("/copy/dir"))),
                ),
                (DirEntry::new(PathBuf::from("/src/a"), FileType::File), None),
                (
                    DirEntry::new(PathBuf::from("/src/link"), FileType::Symlink),
                    None,
                ),
            ] {
                assert_eq!(add_entry(&mut tree, entry, target).unwrap(), expected);
            }

            assert_eq!(tree.dirs.len(), 1);
            assert_eq!(
                tree.files,
                [(PathBuf::from("/src/a"), PathBuf::from("/copy/a"))]
            );
            assert_eq!(
                tree.symlinks,
                [(PathBuf::from("/src/link"), PathBuf::from("/copy/link"))]
            );
        }

        #[test]
        fn add_entry_special_file_should_fail() {
            let mut tree = Tree::default();
            let entry = DirEntry::new(PathBuf::from("/src/socket"), FileType::Other);

            let error = add_entry(&mut tree, entry, Path::new("/copy")).unwrap_err();

            assert_eq!(io_kind(&error), Some(std::io::ErrorKind::Unsupported));
        }
    }

    mod check_remove_dir_all {
        use super::*;

        #[test]
        fn check_remove_dir_all_path_resolving_to_root_should_fail() {
            let test_dir = TestDir::new();
            let depth = test_dir.path().components().count();
            // WHY: only checked, never removed, should the check be broken.
            let root = (1..depth).fold(test_dir.path().to_path_buf(), |path, _| path.join(".."));

            let error = check_remove_dir_all(&root).unwrap_err();

            assert_eq!(io_kind(&error), Some(std::io::ErrorKind::ResourceBusy));
        }
    }

    mod check_not_root {
        use super::*;

        #[test]
        fn check_not_root_should_refuse_only_root() {
            assert!(check_not_root(Path::new("/")).is_err());
            assert!(check_not_root(Path::new("/a")).is_ok());
            assert!(check_not_root(Path::new("a")).is_ok());
        }
    }

    mod resolve_path {
        use super::*;

        #[test]
        fn resolve_path_should_join_relative_path_onto_current_dir() {
            for (path, expected) in [
                ("a", "/cwd/a"),
                ("../a", "/cwd/../a"),
                ("a/", "/cwd/a/"),
                ("/a", "/a"),
            ] {
                assert_eq!(
                    resolve_path(Path::new("/cwd"), Path::new(path)).unwrap(),
                    Path::new(expected),
                    "{path}"
                );
            }
        }

        #[test]
        fn resolve_path_empty_path_should_fail() {
            let error = resolve_path(Path::new("/cwd"), Path::new("")).unwrap_err();

            assert_eq!(io_kind(&error), Some(std::io::ErrorKind::NotFound));
        }
    }

    mod fs_error {
        use super::*;

        #[test]
        fn fs_error_should_describe_with_optional_source() {
            let kind = std::io::ErrorKind::NotFound;

            for (error, description, source) in [
                (fs_error!(NotFound; "a {}", 1), "a 1", Some(kind)),
                (fs_error!(kind => "b {}", 2), "b 2", Some(kind)),
                (fs_error!("c {}", 3), "c 3", None),
            ] {
                assert_eq!(error.kind(), crate::ErrorKind::Fs);
                assert_eq!(error.description(), description);
                assert_eq!(io_kind(&error), source);
            }
        }
    }

    mod fs_error_at {
        use super::*;

        #[test]
        fn fs_error_at_should_append_path_and_keep_source() {
            let error = fs_error_at(Path::new("/a"))(std::io::Error::other("failed"));

            assert_eq!(error.description(), "failed: /a");
            assert_eq!(io_kind(&error), Some(std::io::ErrorKind::Other));
        }
    }

    mod fs_error_between {
        use super::*;

        #[test]
        fn fs_error_between_should_append_both_paths_and_keep_source() {
            let error =
                fs_error_between(Path::new("/a"), Path::new("/b"))(std::io::Error::other("failed"));

            assert_eq!(error.description(), "failed: /a -> /b");
            assert_eq!(io_kind(&error), Some(std::io::ErrorKind::Other));
        }
    }
}
