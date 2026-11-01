use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::ops::Bound;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::sync::RwLock;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::SystemTime;

use crate::Fs;
use crate::ReadFile;
use crate::Result;
use crate::WriteFile;
use crate::env::EnvImpl;
use crate::fs::DirEntry;
use crate::fs::FileType;
use crate::fs::Metadata;
use crate::fs::NativeAttributes;
use crate::fs::OpenWriteOptions;
use crate::fs::check_not_root;
use crate::fs::copy_tree;
#[cfg(unix)]
use crate::fs::executable_mode;
use crate::fs::fs_error_at;
#[cfg(unix)]
use crate::fs::readonly_mode;
use crate::fs::resolve_path;
#[cfg(windows)]
use crate::fs::windows_symlink_target;
pub use crate::test_platform::ROOT;

/// An operation of [`TestFs`] whose failure can be injected, see
/// [`TestFs::set_io_error`].
///
/// Each method of [`Fs`], [`ReadFile`] and [`WriteFile`] is one
/// operation, except that [`Fs::read`], [`Fs::read_to_string`],
/// [`Fs::write`] and [`Fs::copy`] open the file first ([`FsOp::Open`]) and
/// then read or write it, and [`Fs::copy_dir_all`] performs the operations
/// of the methods it is made of.
///
/// # Examples
///
/// ```rust
/// use agplatform::Fs;
/// use agplatform::test_platform::test_fs::{FsOp, TestFs};
/// use std::io::ErrorKind;
///
/// let fs = TestFs::new();
/// fs.set_io_error(FsOp::Write, "/data", ErrorKind::StorageFull);
///
/// assert!(fs.write("/data/a.txt", "hello").is_err());
/// assert!(fs.write("/a.txt", "hello").is_ok());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FsOp {
    /// [`Fs::open_read`], [`Fs::open_write`] and
    /// [`Fs::open_write_options`].
    Open,
    /// [`Read`] and [`ReadFile::read_at`] calls of a handle.
    Read,
    /// [`Write::write`] and [`WriteFile::write_at`] calls of a handle.
    Write,
    /// [`Write::flush`] calls of a handle.
    Flush,
    /// [`Seek::seek`] calls of a handle.
    Seek,
    /// [`WriteFile::set_len`].
    SetLen,
    /// [`WriteFile::sync_all`].
    SyncAll,
    /// [`WriteFile::sync_data`].
    SyncData,
    /// The locking methods of [`ReadFile`] and [`WriteFile`].
    Lock,
    /// [`Fs::metadata`], [`Fs::symlink_metadata`], [`Fs::exists`] and
    /// [`ReadFile::metadata`].
    Metadata,
    /// [`Fs::read_dir`].
    ReadDir,
    /// The entries yielded by the iterator of [`Fs::read_dir`], matched by
    /// their paths. A failed entry is skipped, the iteration continues
    /// with the next one.
    ReadDirEntry,
    /// [`Fs::create_dir`] and [`Fs::create_dir_all`].
    CreateDir,
    /// [`Fs::create_symlink_file`], [`Fs::create_symlink_dir`] and
    /// [`Fs::create_hard_link`].
    CreateLink,
    /// [`Fs::read_link`].
    ReadLink,
    /// [`Fs::canonicalize`].
    Canonicalize,
    /// [`Fs::remove_file`], [`Fs::remove_dir`] and [`Fs::remove_dir_all`].
    Remove,
    /// [`Fs::rename`].
    Rename,
    /// [`Fs::set_readonly`], `Fs::set_executable` and `Fs::set_mode`.
    SetPermissions,
    /// [`Fs::set_modified`].
    SetTimes,
}

/// A fault injected into the matching operations of [`TestFs`].
enum Fault {
    /// One result per call, the last repeated, `None` to succeed.
    Errors(VecDeque<Option<std::io::ErrorKind>>),
    /// Reads or writes of at most the given number of bytes.
    Short(usize),
}

/// A fault set for an operation on the paths starting with `path`.
struct Rule {
    op: FsOp,
    path: PathBuf,
    fault: Fault,
}

/// The faults injected into a [`TestFs`] and its handles.
#[derive(Default)]
struct Faults {
    rules: Vec<Rule>,
    capacity: Option<u64>,
}

impl Faults {
    /// Returns the bytes a read or write `op` on `path` is limited to, if
    /// any, or the error it fails with, advancing the matching sequence.
    fn take(&mut self, op: FsOp, path: &Path) -> std::io::Result<Option<usize>> {
        let Some(rule) = self
            .rules
            .iter_mut()
            .rev()
            .find(|rule| rule.op == op && path.starts_with(&rule.path))
        else {
            return Ok(None);
        };

        match &mut rule.fault {
            Fault::Short(max) => Ok(Some(*max)),
            Fault::Errors(results) => {
                let result = if results.len() > 1 {
                    results.pop_front().flatten()
                } else {
                    results.front().copied().flatten()
                };

                match result {
                    Some(kind) => Err(std::io::Error::new(
                        kind,
                        format!("simulated {op:?} failure"),
                    )),
                    None => Ok(None),
                }
            }
        }
    }
}

/// An in-memory file holding its content, borrow state and metadata.
///
/// Shared by all of its hard links and open handles, see
/// [`TestFsEntry::File`].
#[derive(Debug)]
struct TestFsFile {
    data: Vec<u8>,
    /// At least `data.len()`, the bytes beyond `data` are zeros not held in
    /// memory like in a sparse file.
    len: u64,
    modified: SystemTime,
    created: SystemTime,
    attributes: NativeAttributes,
    state: BorrowState,
    /// The `data` and `len` as of the last sync, which a crash reverts to.
    synced: (Vec<u8>, u64),
}

impl TestFsFile {
    fn new(now: SystemTime, attributes: NativeAttributes) -> Self {
        Self {
            data: Vec::new(),
            len: 0,
            modified: now,
            created: now,
            attributes,
            state: BorrowState::Read(0),
            synced: (Vec::new(), 0),
        }
    }

    fn set_len(&mut self, len: u64) {
        if len < self.data.len() as u64 {
            self.data.truncate(len as usize);
        }

        self.len = len;
        self.modified = SystemTime::now();
    }
}

/// How a file is borrowed by the open handles of [`TestFs`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BorrowState {
    /// Borrowed for reading by the given number of handles.
    Read(u64),
    /// Borrowed for writing by exactly one handle.
    Write,
}

/// An in-memory directory holding its metadata and the names of its
/// children.
struct TestFsDir {
    created: SystemTime,
    modified: SystemTime,
    attributes: NativeAttributes,
    /// The names of the children, each with its [`name_hash`], in the order
    /// [`Fs::read_dir`] lists them.
    children: BTreeSet<(u64, OsString)>,
}

impl TestFsDir {
    fn new(created: SystemTime) -> Self {
        Self {
            created,
            modified: created,
            attributes: new_dir_attributes(),
            children: BTreeSet::new(),
        }
    }
}

/// Returns the 64-bit FNV-1a hash of the file `name`, which orders the
/// entries of a directory deterministically but not alphabetically, like
/// the hashed directories of ext4. Unlike [`std::hash::DefaultHasher`] it is
/// the same across Rust versions.
fn name_hash(name: &OsStr) -> u64 {
    name.as_encoded_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, &byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        })
}

/// An in-memory symlink holding its target as stored and its metadata.
#[derive(Clone)]
struct TestFsSymlink {
    target: PathBuf,
    /// The `target` with `.` and `..` removed lexically, as Windows
    /// resolves it, keeping the leading `..` of a relative one.
    #[cfg(windows)]
    lexical_target: PathBuf,
    /// A directory symlink, which on Windows only [`Fs::remove_dir`]
    /// removes as a directory.
    #[cfg_attr(unix, allow(dead_code))]
    dir: bool,
    created: SystemTime,
    attributes: NativeAttributes,
}

impl TestFsSymlink {
    fn new(target: PathBuf, dir: bool, created: SystemTime) -> Self {
        // WHY: symlink permissions are not used on Linux, their mode is
        // always `0o777` regardless of the umask.
        #[cfg(unix)]
        let attributes = NativeAttributes {
            mode: 0o777,
            uid: OWNER_ID,
            gid: OWNER_ID,
        };
        #[cfg(windows)]
        let attributes = NativeAttributes::default();

        Self {
            #[cfg(windows)]
            lexical_target: crate::utils::resolve_path(Path::new(""), &target),
            target,
            dir,
            created,
            attributes,
        }
    }
}

/// The user and group ID of new entries on Unix.
#[cfg(unix)]
const OWNER_ID: u32 = 1000;

/// The umask new entries are created with on Unix.
#[cfg(unix)]
const UMASK: u32 = 0o022;

/// Returns the attributes of a new file, on Unix with the `mode` of
/// `options` (by default `0o666`), see [`unix_attributes`].
#[cfg(unix)]
fn new_file_attributes(options: &OpenWriteOptions) -> NativeAttributes {
    unix_attributes(options.mode.unwrap_or(0o666))
}

/// Returns the attributes of a new file.
#[cfg(windows)]
fn new_file_attributes(_: &OpenWriteOptions) -> NativeAttributes {
    NativeAttributes::default()
}

/// Returns the attributes of a new directory, on Unix with the mode
/// `0o777`, see [`unix_attributes`].
fn new_dir_attributes() -> NativeAttributes {
    #[cfg(unix)]
    let attributes = unix_attributes(0o777);
    #[cfg(windows)]
    let attributes = NativeAttributes::default();

    attributes
}

/// Returns the attributes of a new Unix entry created with `mode`
/// restricted by [`UMASK`] and owned by [`OWNER_ID`].
#[cfg(unix)]
fn unix_attributes(mode: u32) -> NativeAttributes {
    NativeAttributes {
        mode: mode & 0o7777 & !UMASK,
        uid: OWNER_ID,
        gid: OWNER_ID,
    }
}

/// An entry of the [`TestFs`] tree.
enum TestFsEntry {
    /// A file, shared by all paths that are hard links to it.
    File(Arc<RwLock<TestFsFile>>),
    Dir(TestFsDir),
    Symlink(TestFsSymlink),
}

type Tree = BTreeMap<PathBuf, TestFsEntry>;

/// The state of a [`TestFs`] shared with its handles and directory
/// iterators.
struct Shared {
    tree: Mutex<Tree>,
    faults: Mutex<Faults>,
    /// Incremented by [`TestFs::crash`], which closes the handles opened
    /// before.
    epoch: AtomicU64,
}

impl Shared {
    /// Returns the bytes a read or write `op` on the resolved `path` is
    /// limited to, if any, or its injected error, see [`TestFs::set_io_error`].
    fn inject(&self, op: FsOp, path: &Path) -> std::io::Result<Option<usize>> {
        lock(&self.faults).take(op, path)
    }

    fn capacity(&self) -> Option<u64> {
        lock(&self.faults).capacity
    }
}

/// Fails if growing the `file` from `len` to `new_len` bytes, a new file if
/// `None`, would make the files of the `tree` exceed the `capacity`, see
/// [`TestFs::set_capacity`].
fn check_capacity(
    tree: &Tree,
    capacity: u64,
    file: Option<&Arc<RwLock<TestFsFile>>>,
    len: u64,
    new_len: u64,
) -> std::io::Result<()> {
    if new_len <= len {
        return Ok(());
    }

    let mut counted = HashSet::from([file.map(Arc::as_ptr)]);
    let used = tree
        .values()
        .filter_map(|entry| match entry {
            TestFsEntry::File(other) if counted.insert(Some(Arc::as_ptr(other))) => {
                Some(lock_file_read(other).len)
            }
            _ => None,
        })
        .fold(new_len, u64::saturating_add);

    if used > capacity {
        return Err(std::io::Error::new(
            std::io::ErrorKind::StorageFull,
            "simulated disk full",
        ));
    }

    Ok(())
}

/// The tree locked while a capacity is set, with the capacity, see
/// [`TestFileHandle::lock_write`].
type CapacityLock<'a> = Option<(MutexGuard<'a, Tree>, u64)>;

/// Locks the `mutex`, ignoring a poison: the state is never left
/// inconsistent by a panic.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|error| {
        mutex.clear_poison();
        error.into_inner()
    })
}

/// A purely in-memory filesystem for testing.
///
/// Implements [`Fs`] like [`std::fs`] on the current platform, but stricter:
/// a file can be opened for writing only while no other handle has it open
/// and for reading only while no handle has it open for writing, also by
/// another of its names. A borrowed file cannot be removed or replaced
/// either. Violations fail with [`std::io::ErrorKind::ResourceBusy`], so
/// code passing with it does not depend on sharing that some platforms
/// forbid.
///
/// Failures can be injected per operation and path, see
/// [`set_io_error`](TestFs::set_io_error), as can short reads and writes,
/// a full disk and a crash losing what was not synced.
///
/// # Examples
///
/// ```rust
/// use agplatform::Fs;
/// use agplatform::test_platform::test_fs::TestFs;
///
/// let fs = TestFs::new();
/// fs.write("/greeting.txt", "hello world").unwrap();
/// assert_eq!(fs.read_to_string("/greeting.txt").unwrap(), "hello world");
/// ```
pub struct TestFs {
    env: EnvImpl,
    shared: Arc<Shared>,
    /// The number of symlinks in the tree, changed only while it is locked.
    symlinks: AtomicUsize,
}

impl TestFs {
    /// Creates an in-memory filesystem holding only the [`ROOT`] directory,
    /// which is also the current directory. Other roots, e.g. further
    /// Windows drives, can be added with [`Fs::create_dir_all`].
    ///
    /// On Unix new entries are owned by user and group `1000` and created
    /// under the umask `0o022`: files `0o644`, directories `0o755`.
    ///
    /// Permissions are stored and reported but not modelled as access
    /// checks: only opening a [`readonly`](crate::Metadata::readonly) file
    /// for writing fails. Other mode bits, the owner and the permissions of
    /// directories have no effect, there is no current user. Use
    /// [`set_io_error`](TestFs::set_io_error) to test failing operations.
    ///
    /// Paths are resolved like by the OS: `..` and symlinks component by
    /// component, including relative targets and loops, which fail after
    /// 40 symlinks as on Linux. On Windows `.` and `..` are removed from
    /// the path lexically first and a file as a parent is not found. Symlinks may dangle. The Windows privileges
    /// needed to create symlinks are not modelled. Hard links share the
    /// file itself: its content, metadata and borrows. Opening a directory
    /// fails, as on Windows.
    ///
    /// [`Fs::read_dir`] lists a directory lazily, in an order that is
    /// unspecified as with [`std::fs::read_dir`] but deterministic: by a
    /// fixed hash of the names rather than alphabetically. Files can be
    /// written up to [`MAX_FILE_LEN`] bytes, while lengths set by
    /// [`WriteFile::set_len`] or [`set_len`](TestFs::set_len) extend them
    /// by zeros not held in memory, like sparse files.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::test_platform::test_fs::TestFs;
    ///
    /// let fs = TestFs::new();
    /// ```
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let root = TestFsEntry::Dir(TestFsDir::new(SystemTime::now()));

        TestFs {
            env: EnvImpl::new(Vec::new(), PathBuf::from(ROOT)),
            shared: Arc::new(Shared {
                tree: Mutex::new(BTreeMap::from([(PathBuf::from(ROOT), root)])),
                faults: Mutex::new(Faults::default()),
                epoch: AtomicU64::new(0),
            }),
            symlinks: AtomicUsize::new(0),
        }
    }

    /// Makes the `op` on the paths starting with `path` fail with an
    /// [`std::io::Error`] of `kind`, e.g.
    /// [`StorageFull`](std::io::ErrorKind::StorageFull) for a `write` to a
    /// full disk. The paths are matched with their symlinks resolved, by
    /// their components, an empty `path` matches all of them. A relative
    /// `path` is resolved against the current directory and its `..`
    /// lexically, its symlinks are not resolved. When more
    /// faults match, the most recently set one wins, including those of
    /// [`set_io_errors`](Self::set_io_errors) and
    /// [`set_short_io`](Self::set_short_io), so an empty `path` set first
    /// acts as a default.
    ///
    /// Handles match by the path they were opened at with its symlinks
    /// resolved, so a fault on a symlink does not affect a handle opened
    /// through it, also faults set after they were opened.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, WriteFile};
    /// use agplatform::test_platform::test_fs::{FsOp, TestFs};
    /// use std::io::ErrorKind;
    ///
    /// let fs = TestFs::new();
    /// let file = fs.open_write("/data.db").unwrap();
    /// fs.set_io_error(FsOp::SyncData, "/data.db", ErrorKind::Other);
    ///
    /// assert!(file.sync_data().is_err());
    /// ```
    pub fn set_io_error<P: AsRef<Path>>(&self, op: FsOp, path: P, kind: std::io::ErrorKind) {
        self.add_rule(
            op,
            path.as_ref(),
            Fault::Errors(VecDeque::from([Some(kind)])),
        );
    }

    /// Makes the `op` on the paths starting with `path` succeed (`None`) or
    /// fail with an [`std::io::Error`] of the given kind one call at a time,
    /// the last result repeated, e.g. to fail only the third write. Paths
    /// match as in [`set_io_error`](Self::set_io_error). No `results` set
    /// nothing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Fs;
    /// use agplatform::test_platform::test_fs::{FsOp, TestFs};
    /// use std::io::ErrorKind;
    ///
    /// let fs = TestFs::new();
    /// fs.set_io_errors(FsOp::Open, "", [None, Some(ErrorKind::Interrupted), None]);
    ///
    /// assert!(fs.write("/a.txt", "a").is_ok());
    /// assert!(fs.write("/a.txt", "b").is_err());
    /// assert!(fs.write("/a.txt", "c").is_ok());
    /// ```
    pub fn set_io_errors<P: AsRef<Path>, I: IntoIterator<Item = Option<std::io::ErrorKind>>>(
        &self,
        op: FsOp,
        path: P,
        results: I,
    ) {
        let results: VecDeque<_> = results.into_iter().collect();

        if !results.is_empty() {
            self.add_rule(op, path.as_ref(), Fault::Errors(results));
        }
    }

    /// Makes each [`FsOp::Read`] or [`FsOp::Write`] call on the paths
    /// starting with `path` handle at most `max` bytes, as reads and writes
    /// may, e.g. to test that a caller retries the rest. Other operations
    /// are not affected. Paths match as in
    /// [`set_io_error`](Self::set_io_error).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Fs;
    /// use agplatform::test_platform::test_fs::{FsOp, TestFs};
    /// use std::io::Write;
    ///
    /// let fs = TestFs::new();
    /// fs.set_short_io(FsOp::Write, "", 2);
    ///
    /// let mut file = fs.open_write("/data.txt").unwrap();
    /// assert_eq!(file.write(b"hello").unwrap(), 2);
    /// file.write_all(b"llo").unwrap();
    /// drop(file);
    /// assert_eq!(fs.read_to_string("/data.txt").unwrap(), "hello");
    /// ```
    pub fn set_short_io<P: AsRef<Path>>(&self, op: FsOp, path: P, max: usize) {
        self.add_rule(op, path.as_ref(), Fault::Short(max));
    }

    /// Limits the total length of the files to `capacity` bytes, `None` for
    /// no limit (the default). Writing or extending a file beyond it fails
    /// with [`StorageFull`](std::io::ErrorKind::StorageFull), as on a full
    /// disk. Files already beyond it are kept and can still shrink.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Fs;
    /// use agplatform::test_platform::test_fs::TestFs;
    ///
    /// let fs = TestFs::new();
    /// fs.set_capacity(Some(8));
    /// fs.write("/a.txt", "hello").unwrap();
    ///
    /// assert!(fs.write("/b.txt", "world").is_err());
    /// ```
    pub fn set_capacity(&self, capacity: Option<u64>) {
        lock(&self.shared.faults).capacity = capacity;
    }

    /// Removes all faults set by [`set_io_error`](Self::set_io_error),
    /// [`set_io_errors`](Self::set_io_errors),
    /// [`set_short_io`](Self::set_short_io) and
    /// [`set_capacity`](Self::set_capacity).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Fs;
    /// use agplatform::test_platform::test_fs::{FsOp, TestFs};
    /// use std::io::ErrorKind;
    ///
    /// let fs = TestFs::new();
    /// fs.set_io_error(FsOp::Open, "", ErrorKind::Other);
    /// fs.clear_faults();
    ///
    /// assert!(fs.write("/a.txt", "a").is_ok());
    /// ```
    pub fn clear_faults(&self) {
        *lock(&self.shared.faults) = Faults::default();
    }

    /// Simulates a crash of the system: every file loses what was written
    /// to it since it was last synced by [`WriteFile::sync_data`],
    /// [`WriteFile::sync_all`] or [`sync`](Self::sync), its content and
    /// length revert to what they were then, empty for a file never synced.
    /// All open handles are closed: their calls fail and the files can be
    /// opened again.
    ///
    /// Directory entries are durable right away: files and directories
    /// created, renamed or removed stay so, as on filesystems journaling
    /// their metadata.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::{Fs, WriteFile};
    /// use agplatform::test_platform::test_fs::TestFs;
    /// use std::io::Write;
    ///
    /// let fs = TestFs::new();
    /// let mut wal = fs.open_write("/wal").unwrap();
    /// wal.write_all(b"commit 1;").unwrap();
    /// wal.sync_data().unwrap();
    /// wal.write_all(b"commit 2;").unwrap();
    ///
    /// fs.crash();
    ///
    /// assert!(wal.write_all(b"commit 3;").is_err());
    /// assert_eq!(fs.read_to_string("/wal").unwrap(), "commit 1;");
    /// ```
    pub fn crash(&self) {
        let tree = lock(&self.shared.tree);
        self.shared.epoch.fetch_add(1, Ordering::SeqCst);

        for entry in tree.values() {
            if let TestFsEntry::File(file) = entry {
                let mut file = lock_file_write(file);
                file.data = file.synced.0.clone();
                file.len = file.synced.1;
                file.state = BorrowState::Read(0);
            }
        }
    }

    /// Marks the content of every file as synced, as if
    /// [`WriteFile::sync_all`] was called on each, e.g. once a test has set
    /// up the files that should survive a [`crash`](Self::crash).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Fs;
    /// use agplatform::test_platform::test_fs::TestFs;
    ///
    /// let fs = TestFs::new();
    /// fs.write("/config.toml", "a = 1").unwrap();
    /// fs.sync();
    ///
    /// fs.crash();
    ///
    /// assert_eq!(fs.read_to_string("/config.toml").unwrap(), "a = 1");
    /// ```
    pub fn sync(&self) {
        for entry in lock(&self.shared.tree).values() {
            if let TestFsEntry::File(file) = entry {
                let mut file = lock_file_write(file);
                file.synced = (file.data.clone(), file.len);
            }
        }
    }

    /// Sets the creation time of a file or directory as reported by
    /// [`Fs::metadata`]. Unlike [`Fs::set_modified`] this is not
    /// available on real filesystems.
    ///
    /// # Errors
    ///
    /// Returns an error if the path does not exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Fs;
    /// use agplatform::test_platform::test_fs::TestFs;
    /// use std::time::SystemTime;
    ///
    /// let fs = TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// fs.set_created("/data.txt", SystemTime::UNIX_EPOCH).unwrap();
    /// assert_eq!(fs.metadata("/data.txt").unwrap().created, SystemTime::UNIX_EPOCH);
    /// ```
    pub fn set_created<P: AsRef<Path>>(&self, path: P, created: SystemTime) -> Result {
        let (mut tree, normalized_path, resolved_path) = self.lock_resolve(path.as_ref(), true)?;

        match tree.get_mut(&resolved_path) {
            Some(TestFsEntry::File(file)) => lock_file_write(file).created = created,
            Some(TestFsEntry::Dir(dir)) => dir.created = created,
            Some(TestFsEntry::Symlink(_)) | None => {
                return Err(fs_error!(
                    missing_kind(&tree, &resolved_path) =>
                    "Creation time could not be set, path not found: {}",
                    normalized_path.display()
                ));
            }
        }

        Ok(())
    }

    /// Sets the length of a file like [`WriteFile::set_len`] without
    /// opening it, i.e. regardless of its borrows, permissions, faults and
    /// the capacity. Its content is cut at `len` or extended by zeros that
    /// are not held in memory, so a file can be made arbitrarily large.
    /// Symlinks are followed.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The path does not exist or is a directory.
    /// - `len` exceeds `i64::MAX`, like [`WriteFile::set_len`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Fs;
    /// use agplatform::test_platform::test_fs::TestFs;
    ///
    /// let fs = TestFs::new();
    /// fs.write("/image.iso", "").unwrap();
    /// fs.set_len("/image.iso", 10 << 30).unwrap();
    /// assert_eq!(fs.metadata("/image.iso").unwrap().len, 10 << 30);
    /// ```
    pub fn set_len<P: AsRef<Path>>(&self, path: P, len: u64) -> Result {
        let (mut tree, normalized_path, resolved_path) = self.lock_resolve(path.as_ref(), true)?;

        match tree.get_mut(&resolved_path) {
            Some(TestFsEntry::File(_)) if i64::try_from(len).is_err() => {
                Err(invalid_len_error(&normalized_path))
            }
            Some(TestFsEntry::File(file)) => {
                lock_file_write(file).set_len(len);
                Ok(())
            }
            Some(TestFsEntry::Dir(_)) => Err(fs_error!(
                IsADirectory;
                "File length could not be set, it is a directory: {}",
                normalized_path.display()
            )),
            Some(TestFsEntry::Symlink(_)) | None => Err(fs_error!(
                missing_kind(&tree, &resolved_path) =>
                "File length could not be set, path not found: {}",
                normalized_path.display()
            )),
        }
    }

    /// Sets the owner of a file or directory as reported by
    /// [`Fs::metadata`]. Unlike [`Fs::set_mode`] this is not available on
    /// real filesystems, changing the owner requires privileges.
    ///
    /// Unix only: std does not report owners on Windows.
    ///
    /// # Errors
    ///
    /// Returns an error if the path does not exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use agplatform::Fs;
    /// use agplatform::test_platform::test_fs::TestFs;
    ///
    /// let fs = TestFs::new();
    /// fs.write("/data.txt", "hello").unwrap();
    /// fs.set_owner("/data.txt", 0, 0).unwrap();
    /// assert_eq!(fs.metadata("/data.txt").unwrap().uid, 0);
    /// ```
    #[cfg(unix)]
    pub fn set_owner<P: AsRef<Path>>(&self, path: P, uid: u32, gid: u32) -> Result {
        self.update_attributes(path.as_ref(), |attributes| {
            attributes.uid = uid;
            attributes.gid = gid;
        })
    }

    /// Sets or clears the [`hidden`](crate::Metadata::hidden) attribute of a
    /// file or directory, which `FsImpl` cannot set as std does not support
    /// it.
    ///
    /// Windows only: on Unix a file is hidden when its name starts with
    /// `.`.
    ///
    /// # Errors
    ///
    /// Returns an error if the path does not exist.
    #[cfg(windows)]
    pub fn set_hidden<P: AsRef<Path>>(&self, path: P, hidden: bool) -> Result {
        self.update_attributes(path.as_ref(), |attributes| {
            attributes.hidden = hidden;
        })
    }

    /// [internal]
    ///
    /// Resolves relative paths against the current directory of `env`
    /// instead, e.g. of a [`TestPlatform`](crate::test_platform::TestPlatform).
    pub(crate) fn set_env(&mut self, env: EnvImpl) {
        self.env = env;
    }

    /// [internal]
    ///
    /// Returns the current directory relative paths are resolved against.
    #[cfg(test)]
    pub(crate) fn current_dir(&self) -> PathBuf {
        self.env.current_dir_arc().to_path_buf()
    }

    fn add_rule(&self, op: FsOp, path: &Path, fault: Fault) {
        let path = if path.as_os_str().is_empty() {
            PathBuf::new()
        } else {
            // WHY: lexically, the rule may be set before the path exists.
            self.env.current_dir_arc().join(path).components().fold(
                PathBuf::new(),
                |mut path, component| {
                    if component == Component::ParentDir {
                        path.pop();
                    } else {
                        path.push(component);
                    }
                    path
                },
            )
        };

        lock(&self.shared.faults)
            .rules
            .push(Rule { op, path, fault });
    }

    /// Fails with the error injected into the `op` on the `resolved` path,
    /// if any, reported for the `normalized_path` it was resolved from.
    fn inject(&self, op: FsOp, resolved: &Path, normalized_path: &Path) -> Result {
        self.shared
            .inject(op, resolved)
            .map(|_| ())
            .map_err(fs_error_at(normalized_path))
    }

    /// Updates the attributes of the file or directory at `path`, followed
    /// if it is a symlink, with `update`.
    fn update_attributes(&self, path: &Path, update: impl FnOnce(&mut NativeAttributes)) -> Result {
        let (mut tree, normalized_path, resolved_path) = self.lock_resolve(path, true)?;

        match tree.get_mut(&resolved_path) {
            Some(TestFsEntry::File(file)) => update(&mut lock_file_write(file).attributes),
            Some(TestFsEntry::Dir(dir)) => update(&mut dir.attributes),
            Some(TestFsEntry::Symlink(_)) | None => {
                return Err(fs_error!(
                    missing_kind(&tree, &resolved_path) =>
                    "Attributes could not be set, path not found: {}",
                    normalized_path.display()
                ));
            }
        }

        Ok(())
    }

    /// Locks the tree and returns it with the normalized `path`, for
    /// messages, and the `path` with its symlinks resolved, the last
    /// component only if `follow_last` is set, see
    /// [`resolve_dir`](Self::resolve_dir).
    fn lock_resolve(
        &self,
        path: &Path,
        follow_last: bool,
    ) -> Result<(MutexGuard<'_, Tree>, PathBuf, PathBuf)> {
        let (tree, normalized_path) = self.lock_normalize(path)?;
        let resolved_path = self.resolve_dir(&tree, &normalized_path, follow_last)?;

        Ok((tree, normalized_path, resolved_path))
    }

    /// Locks the tree and returns it with the normalized `path`.
    fn lock_normalize(&self, path: &Path) -> Result<(MutexGuard<'_, Tree>, PathBuf)> {
        self.lock_normalize_at(&self.env.current_dir_arc(), path)
    }

    /// Locks the tree and returns it with the `path` normalized against
    /// `current_dir`, see [`normalize_at`].
    fn lock_normalize_at(
        &self,
        current_dir: &Path,
        path: &Path,
    ) -> Result<(MutexGuard<'_, Tree>, PathBuf)> {
        let normalized_path = normalize_at(current_dir, path)?;
        Ok((lock(&self.shared.tree), normalized_path))
    }

    /// Resolves the `..` and the symlinks of a `path` normalized by
    /// [`normalize_at`], see [`resolve_symlinks`]. While the tree has no
    /// symlinks a `path` without `..` is returned without walking the
    /// tree. A trailing separator is dropped, see
    /// [`resolve_dir`](Self::resolve_dir) to honour it.
    fn resolve(&self, tree: &Tree, path: &Path, follow_last: bool) -> Result<PathBuf> {
        // WHY: relaxed suffices, the count only changes while the tree is
        // locked, which the caller holds.
        if self.symlinks.load(Ordering::Relaxed) == 0
            && !path
                .components()
                .any(|component| component == Component::ParentDir)
        {
            return Ok(path.components().collect());
        }

        resolve_symlinks(tree, path, follow_last)
    }

    /// Resolves a normalized `path` like [`resolve`](Self::resolve) unless
    /// it [`requires_dir`] or [`ends_with_current_dir`]: then its last
    /// component is followed and fails if it is not a directory.
    fn resolve_dir(&self, tree: &Tree, path: &Path, follow_last: bool) -> Result<PathBuf> {
        #[cfg(unix)]
        let dir_required = requires_dir(path) || ends_with_current_dir(path);
        #[cfg(windows)]
        let dir_required = requires_dir(path);
        let resolved = self.resolve(tree, path, follow_last || dir_required)?;

        if dir_required && matches!(tree.get(&resolved), Some(TestFsEntry::File(_))) {
            return Err(fs_error!(
                NotADirectory;
                "Not a directory: {}",
                path.display()
            ));
        }

        Ok(resolved)
    }

    /// Fails like the OS if the normalized `path` to be created
    /// [`ends_with_current_dir`] but is not an existing directory.
    #[cfg(unix)]
    fn check_current_dir(&self, tree: &Tree, path: &Path, what: &str) -> Result {
        if !ends_with_current_dir(path) {
            return Ok(());
        }

        let resolved = self.resolve(tree, path, true)?;

        match tree.get(&resolved) {
            Some(TestFsEntry::Dir(_)) => Ok(()),
            Some(TestFsEntry::File(_)) => Err(fs_error!(
                NotADirectory;
                "{what}, path is not a directory: {}",
                path.display()
            )),
            Some(TestFsEntry::Symlink(_)) | None => Err(fs_error!(
                missing_kind(tree, &resolved) =>
                "{what}, path not found: {}",
                path.display()
            )),
        }
    }

    /// Returns a handle of the `file` resolved at `resolved` from the
    /// normalized `path`.
    fn handle(
        &self,
        file: &Arc<RwLock<TestFsFile>>,
        path: PathBuf,
        resolved: PathBuf,
    ) -> TestFileHandle {
        TestFileHandle {
            pos: AtomicU64::new(0),
            file: file.clone(),
            path,
            resolved,
            shared: self.shared.clone(),
            epoch: self.shared.epoch.load(Ordering::SeqCst),
        }
    }
}

/// Returns `true` if `path` ends with a separator, which [`Path::components`]
/// drops but which requires the path to be a directory.
fn ends_with_separator(path: &Path) -> bool {
    path.as_os_str()
        .as_encoded_bytes()
        .last()
        .is_some_and(|&byte| std::path::is_separator(byte.into()))
}

/// Returns `true` if the normalized `path` ends with a separator other
/// than a root, so that it must be a directory.
fn requires_dir(path: &Path) -> bool {
    ends_with_separator(path) && path.parent().is_some()
}

/// Returns `true` if `path` ends with a separator and `.`, which
/// [`Path::components`] drops but which requires the path to be an
/// existing directory. Never on Windows, whose normalized paths have no
/// `.`.
#[cfg(unix)]
fn ends_with_current_dir(path: &Path) -> bool {
    match path.as_os_str().as_encoded_bytes() {
        [.., separator, b'.'] => std::path::is_separator(char::from(*separator)),
        _ => false,
    }
}

/// Returns the kind of error for the missing resolved `path`: not a
/// directory if its closest existing ancestor is not one, as reported by
/// the OS, not found otherwise.
fn missing_kind(tree: &Tree, path: &Path) -> std::io::ErrorKind {
    match path
        .ancestors()
        .skip(1)
        .find_map(|ancestor| tree.get(ancestor))
    {
        Some(TestFsEntry::Dir(_)) | None => std::io::ErrorKind::NotFound,
        Some(_) => std::io::ErrorKind::NotADirectory,
    }
}

/// Returns the error for reading a path that is not a symlink as one, the
/// OS error `EINVAL`, on Windows `ERROR_NOT_A_REPARSE_POINT`, which std
/// reports as uncategorized.
fn not_symlink_error() -> std::io::Error {
    #[cfg(unix)]
    const NOT_SYMLINK: i32 = 22;
    #[cfg(windows)]
    const NOT_SYMLINK: i32 = 4390;

    std::io::Error::from_raw_os_error(NOT_SYMLINK)
}

/// Returns the error for resolving a path through more than
/// [`MAX_SYMLINKS`] symlinks, the OS error `ELOOP` which std reports as a
/// filesystem loop.
fn symlink_loop_error() -> std::io::Error {
    #[cfg(target_os = "linux")]
    const ELOOP: i32 = 40;
    #[cfg(all(unix, not(target_os = "linux")))]
    const ELOOP: i32 = 62;
    // WHY: `ERROR_CANT_RESOLVE_FILENAME`, which std maps to the same kind.
    #[cfg(windows)]
    const ELOOP: i32 = 1921;

    std::io::Error::from_raw_os_error(ELOOP)
}

/// The number of symlinks followed when resolving a path before it is
/// considered a loop, `MAXSYMLINKS` on Linux.
const MAX_SYMLINKS: usize = 40;

/// Returns `path` joined onto `current_dir`, see [`resolve_path`], on
/// Windows with `.` and `..` removed. Fails if it contains a NUL byte,
/// which the OS rejects.
///
/// WHY: the two paths of an operation are normalized against the same
/// current directory, even when it is changed in between.
fn normalize_at(current_dir: &Path, path: &Path) -> Result<PathBuf> {
    if path.as_os_str().as_encoded_bytes().contains(&0) {
        return Err(fs_error!(
            InvalidInput;
            "Path contains a NUL byte: {}",
            path.display()
        ));
    }

    let path = resolve_path(current_dir, path)?.into_owned();

    // WHY: Windows removes `.` and `..` lexically before resolving
    // symlinks, keeping a trailing separator.
    #[cfg(windows)]
    let path = {
        let mut lexical = crate::utils::resolve_path(Path::new(""), &path);
        if ends_with_separator(&path) && lexical.parent().is_some() {
            lexical
                .as_mut_os_string()
                .push(std::path::MAIN_SEPARATOR_STR);
        }
        lexical
    };

    Ok(path)
}

/// Resolves the `..` and the symlinks of a normalized `path`: in the parent
/// components always, in the last component only if `follow_last` is set.
/// A relative target continues from the directory containing the symlink
/// and a `..` leaves the directory the path has resolved into so far, which
/// must exist and be a directory, as the OS requires.
fn resolve_symlinks(tree: &Tree, path: &Path, follow_last: bool) -> Result<PathBuf> {
    let mut components: VecDeque<Component> = path.components().collect();
    let mut resolved = PathBuf::new();
    let mut followed = 0;

    while let Some(component) = components.pop_front() {
        match component {
            // WHY: on Windows the paths and lexical targets have no `.`.
            #[cfg(unix)]
            Component::CurDir => {}
            Component::ParentDir => {
                // WHY: on Windows only the leading `..` of a lexical symlink
                // target are left, which leave the directory of the symlink.
                #[cfg(unix)]
                if !is_dir(tree, &resolved) {
                    let kind = match tree.get(&resolved) {
                        Some(_) => std::io::ErrorKind::NotADirectory,
                        None => missing_kind(tree, &resolved),
                    };

                    return Err(fs_error!(
                        kind =>
                        "Path could not be resolved, not a directory: {}",
                        resolved.display()
                    ));
                }

                resolved.pop();
            }
            _ => resolved.push(component),
        }

        if let Some(TestFsEntry::Symlink(symlink)) = tree.get(&resolved)
            && (follow_last || !components.is_empty())
        {
            followed += 1;

            if followed > MAX_SYMLINKS {
                return Err(
                    fs_error!("Too many levels of symbolic links: {}", path.display())
                        .with_source(symlink_loop_error()),
                );
            }

            resolved.pop();
            #[cfg(unix)]
            let target = &symlink.target;
            #[cfg(windows)]
            let target = &symlink.lexical_target;

            for target_component in target.components().rev() {
                components.push_front(target_component);
            }
        }
    }

    Ok(resolved)
}

/// The state shared by [`TestFileRead`] and [`TestFileWrite`].
struct TestFileHandle {
    /// The position, atomic so that positional calls on `&self` can move
    /// it as on Windows.
    pos: AtomicU64,
    file: Arc<RwLock<TestFsFile>>,
    /// The path the file was opened at, resolved against the current
    /// directory.
    path: PathBuf,
    /// The `path` with its symlinks resolved, which faults match.
    resolved: PathBuf,
    shared: Arc<Shared>,
    /// The [`Shared::epoch`] the handle was opened in.
    epoch: u64,
}

impl TestFileHandle {
    /// Fails if the [`TestFs`] crashed since the handle was opened.
    fn check_open(&self) -> std::io::Result<()> {
        if self.shared.epoch.load(Ordering::SeqCst) != self.epoch {
            return Err(std::io::Error::other(
                "TestFs crashed, the file handle is closed",
            ));
        }

        Ok(())
    }

    /// Fails if the handle is closed or the `op` fails, see
    /// [`Shared::inject`].
    fn inject(&self, op: FsOp) -> std::io::Result<Option<usize>> {
        self.check_open()?;
        self.faults(op)
    }

    /// Like [`inject`](Self::inject) without checking the handle, for the
    /// operations that check it once they locked the file, so that a
    /// [`TestFs::crash`] either reverts them or closes the handle first.
    fn faults(&self, op: FsOp) -> std::io::Result<Option<usize>> {
        self.shared.inject(op, &self.resolved)
    }

    /// Like [`inject`](Self::inject) for the methods of [`ReadFile`] and
    /// [`WriteFile`], which report the path.
    fn check(&self, op: FsOp) -> Result {
        self.inject(op).map(|_| ()).map_err(fs_error_at(&self.path))
    }

    fn len(&self) -> u64 {
        lock_file_read(&self.file).len
    }

    fn metadata(&self) -> Result<Metadata> {
        self.check(FsOp::Metadata)?;
        let file = lock_file_read(&self.file);

        Ok(Metadata::new(
            &self.path,
            FileType::File,
            file.len,
            file.modified,
            file.created,
            file.attributes,
        ))
    }

    /// Reads into `buf` from `pos` and returns the number of bytes read.
    fn read_at(&self, buf: &mut [u8], pos: u64) -> std::io::Result<usize> {
        let max = self.faults(FsOp::Read)?.unwrap_or(usize::MAX);
        let buf_len = buf.len().min(max);

        self.read_with(pos, buf_len as u64, |held, zeroes| {
            buf[..held.len()].copy_from_slice(held);
            buf[held.len()..held.len() + zeroes].fill(0);
            Ok(())
        })
    }

    /// Reads up to `max` bytes from `pos`, passing `read` the bytes held of
    /// them and the number of the zero bytes past those of a sparse file,
    /// and returns their total.
    fn read_with(
        &self,
        pos: u64,
        max: u64,
        read: impl FnOnce(&[u8], usize) -> std::io::Result<()>,
    ) -> std::io::Result<usize> {
        let file = lock_file_read(&self.file);
        self.check_open()?;
        let read_size = file.len.saturating_sub(pos).min(max).min(usize::MAX as u64) as usize;
        let start = std::cmp::min(pos, file.data.len() as u64) as usize;
        let held = std::cmp::min(file.data.len() - start, read_size);
        read(&file.data[start..start + held], read_size - held)?;
        Ok(read_size)
    }

    /// Reads into `buf` from the position and advances it.
    fn read(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        let pos = self.pos.load(Ordering::SeqCst);
        let read = self.read_at(buf, pos)?;
        self.pos.store(pos + read as u64, Ordering::SeqCst);
        Ok(read)
    }

    // WHY: read at once rather than in growing chunks, each zeroed first,
    // unless short reads are injected.
    fn read_to_end(&self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut total = 0;

        loop {
            let max = self.faults(FsOp::Read)?.map_or(u64::MAX, |max| max as u64);
            let pos = self.pos.load(Ordering::SeqCst);
            let read = self.read_with(pos, max, |held, zeroes| {
                buf.try_reserve(held.len() + zeroes)
                    .map_err(|_| std::io::Error::from(std::io::ErrorKind::OutOfMemory))?;
                buf.extend_from_slice(held);
                buf.resize(buf.len() + zeroes, 0);
                Ok(())
            })?;
            self.pos.store(pos + read as u64, Ordering::SeqCst);
            total += read;

            if read == 0 || (read as u64) < max {
                return Ok(total);
            }
        }
    }

    /// Writes `buf` at `pos`, the end of the file if `append`, and returns
    /// the number of bytes written and the position after them.
    fn write_at(&self, buf: &[u8], pos: u64, append: bool) -> std::io::Result<(usize, u64)> {
        let max = self.faults(FsOp::Write)?.unwrap_or(usize::MAX);
        let buf = &buf[..buf.len().min(max)];
        let (tree, mut file) = self.lock_write()?;
        let pos = if append { file.len } else { pos };

        if buf.is_empty() {
            return Ok((0, pos));
        }

        let end_pos = pos
            .checked_add(buf.len() as u64)
            .filter(|&end_pos| end_pos <= MAX_FILE_LEN)
            .ok_or_else(file_too_large_error)?;
        self.check_capacity(&tree, &file, end_pos)?;

        if end_pos > file.data.len() as u64 {
            file.data.resize(end_pos as usize, 0);
        }

        file.data[pos as usize..end_pos as usize].copy_from_slice(buf);
        file.len = file.len.max(end_pos);
        file.modified = SystemTime::now();

        Ok((buf.len(), end_pos))
    }

    /// Locks the file for writing and fails if the handle was closed by a
    /// crash meanwhile. While a capacity is set the tree is locked first,
    /// so that growing the file is checked and done at once, see
    /// [`check_capacity`](Self::check_capacity).
    fn lock_write(
        &self,
    ) -> std::io::Result<(
        CapacityLock<'_>,
        std::sync::RwLockWriteGuard<'_, TestFsFile>,
    )> {
        let tree = self
            .shared
            .capacity()
            .map(|capacity| (lock(&self.shared.tree), capacity));
        let file = lock_file_write(&self.file);
        self.check_open()?;
        Ok((tree, file))
    }

    /// Fails if growing the `file` locked by [`lock_write`](Self::lock_write)
    /// to `new_len` bytes would exceed the capacity, see [`check_capacity`].
    fn check_capacity(
        &self,
        tree: &CapacityLock<'_>,
        file: &TestFsFile,
        new_len: u64,
    ) -> std::io::Result<()> {
        match tree {
            Some((tree, capacity)) => {
                check_capacity(tree, *capacity, Some(&self.file), file.len, new_len)
            }
            None => Ok(()),
        }
    }

    /// Marks the content of the file as synced, see [`TestFs::crash`].
    fn sync(&self, op: FsOp) -> Result {
        self.faults(op).map_err(fs_error_at(&self.path))?;
        let mut file = lock_file_write(&self.file);
        self.check_open().map_err(fs_error_at(&self.path))?;
        file.synced = (file.data.clone(), file.len);
        Ok(())
    }

    fn seek(&self, pos: SeekFrom) -> std::io::Result<u64> {
        self.inject(FsOp::Seek)?;

        // WHY: like `lseek`, a position before the start or beyond
        // `i64::MAX` is invalid.
        let new_pos = match pos {
            SeekFrom::Start(offset) => i64::try_from(offset).ok(),
            SeekFrom::End(offset) => i64::try_from(self.len())
                .ok()
                .and_then(|len| len.checked_add(offset)),
            SeekFrom::Current(offset) => i64::try_from(self.pos.load(Ordering::SeqCst))
                .ok()
                .and_then(|pos| pos.checked_add(offset)),
        };
        let new_pos = new_pos
            .and_then(|pos| u64::try_from(pos).ok())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "invalid seek to a negative or overflowing position",
                )
            })?;
        self.pos.store(new_pos, Ordering::SeqCst);

        Ok(new_pos)
    }

    // WHY: within one `TestFs` the borrow rules exclude handles with
    // conflicting locks, a write handle is the only one of its file, so
    // every lock is acquired right away.
    fn lock(&self) -> Result<bool> {
        self.check(FsOp::Lock).map(|()| true)
    }
}

impl Drop for TestFileHandle {
    fn drop(&mut self) {
        let mut file = lock_file_write(&self.file);

        // WHY: a crash released the borrows of the handles opened before.
        if self.shared.epoch.load(Ordering::SeqCst) != self.epoch {
            return;
        }

        match &mut file.state {
            BorrowState::Read(count) => {
                *count = count.saturating_sub(1);
            }
            BorrowState::Write => {
                file.state = BorrowState::Read(0);
            }
        }
    }
}

/// The largest position up to which a [`TestFs`] file can be written,
/// beyond which writing fails as if the file were too large for the
/// filesystem rather than allocating it in memory. Lengths set by
/// [`WriteFile::set_len`] or [`TestFs::set_len`] are not limited, the bytes
/// beyond the written ones are zeros not held in memory.
pub const MAX_FILE_LEN: u64 = 1 << 30;

/// Returns the error for writing a file beyond [`MAX_FILE_LEN`].
fn file_too_large_error() -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::FileTooLarge,
        format!("file would exceed the maximum length of {MAX_FILE_LEN} bytes"),
    )
}

/// Returns the error for a length `ftruncate` rejects, which takes a
/// signed length.
fn invalid_len_error(path: &Path) -> crate::Error {
    fs_error!(
        InvalidInput;
        "File length could not be set, length is invalid: {}",
        path.display()
    )
}

fn lock_file_read(file: &RwLock<TestFsFile>) -> std::sync::RwLockReadGuard<'_, TestFsFile> {
    file.read().unwrap_or_else(|error| {
        file.clear_poison();
        error.into_inner()
    })
}

fn lock_file_write(file: &RwLock<TestFsFile>) -> std::sync::RwLockWriteGuard<'_, TestFsFile> {
    file.write().unwrap_or_else(|error| {
        file.clear_poison();
        error.into_inner()
    })
}

/// A read-only handle to an in-memory file in [`TestFs`]. Dropping the
/// handle releases the read borrow.
pub struct TestFileRead(TestFileHandle);

impl Read for TestFileRead {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.0.read_to_end(buf)
    }
}

impl Seek for TestFileRead {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.0.seek(pos)
    }
}

impl ReadFile for TestFileRead {
    fn path(&self) -> &Path {
        &self.0.path
    }

    fn metadata(&self) -> Result<Metadata> {
        self.0.metadata()
    }

    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        let read = self
            .0
            .read_at(buf, offset)
            .map_err(fs_error_at(&self.0.path))?;
        self.0.pos.store(offset + read as u64, Ordering::SeqCst);
        Ok(read)
    }

    fn lock_shared(&self) -> Result {
        self.0.lock().map(|_| ())
    }

    fn try_lock_shared(&self) -> Result<bool> {
        self.0.lock()
    }

    fn unlock(&self) -> Result {
        self.0.lock().map(|_| ())
    }
}

/// A read-write handle to an in-memory file in [`TestFs`]. Dropping the
/// handle releases the write borrow.
pub struct TestFileWrite {
    handle: TestFileHandle,
    append: bool,
    write_only: bool,
}

impl TestFileWrite {
    fn check_readable(&self) -> std::io::Result<()> {
        if self.write_only {
            return Err(std::io::Error::other("file is opened for writing only"));
        }

        Ok(())
    }
}

impl Read for TestFileWrite {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.check_readable()?;
        self.handle.read(buf)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.check_readable()?;
        self.handle.read_to_end(buf)
    }
}

impl Seek for TestFileWrite {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.handle.seek(pos)
    }
}

impl Write for TestFileWrite {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let pos = self.handle.pos.load(Ordering::SeqCst);
        let (written, end_pos) = self.handle.write_at(buf, pos, self.append)?;
        self.handle.pos.store(end_pos, Ordering::SeqCst);
        Ok(written)
    }

    // WHY: written as one operation like a vectored write of the OS, e.g.
    // all of a record appended to a log or none of it on a crash.
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.write(
            &bufs
                .iter()
                .flat_map(|buf| buf.iter().copied())
                .collect::<Vec<_>>(),
        )
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.handle.inject(FsOp::Flush).map(|_| ())
    }
}

impl ReadFile for TestFileWrite {
    fn path(&self) -> &Path {
        &self.handle.path
    }

    fn metadata(&self) -> Result<Metadata> {
        self.handle.metadata()
    }

    fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize> {
        let read = self
            .check_readable()
            .and_then(|()| self.handle.read_at(buf, offset))
            .map_err(fs_error_at(&self.handle.path))?;
        self.handle
            .pos
            .store(offset + read as u64, Ordering::SeqCst);
        Ok(read)
    }

    fn lock_shared(&self) -> Result {
        self.handle.lock().map(|_| ())
    }

    fn try_lock_shared(&self) -> Result<bool> {
        self.handle.lock()
    }

    fn unlock(&self) -> Result {
        self.handle.lock().map(|_| ())
    }
}

impl WriteFile for TestFileWrite {
    fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize> {
        let (written, end_pos) = self
            .handle
            .write_at(buf, offset, self.append)
            .map_err(fs_error_at(&self.handle.path))?;
        self.handle.pos.store(end_pos, Ordering::SeqCst);
        Ok(written)
    }

    fn set_len(&self, len: u64) -> Result {
        self.handle
            .faults(FsOp::SetLen)
            .map_err(fs_error_at(&self.handle.path))?;

        if i64::try_from(len).is_err() {
            return Err(invalid_len_error(&self.handle.path));
        }

        let (tree, mut file) = self
            .handle
            .lock_write()
            .map_err(fs_error_at(&self.handle.path))?;
        self.handle
            .check_capacity(&tree, &file, len)
            .map_err(fs_error_at(&self.handle.path))?;
        file.set_len(len);
        Ok(())
    }

    fn sync_all(&self) -> Result {
        self.handle.sync(FsOp::SyncAll)
    }

    fn sync_data(&self) -> Result {
        self.handle.sync(FsOp::SyncData)
    }

    fn lock(&self) -> Result {
        self.handle.lock().map(|_| ())
    }

    fn try_lock(&self) -> Result<bool> {
        self.handle.lock()
    }
}

/// A lazy iterator over the entries of a directory of [`TestFs`] in the
/// order of the FNV-1a hashes of their names.
///
/// Like [`std::fs::ReadDir`] it does not hold the filesystem: each entry is
/// looked up when it is reached, continuing after the last one returned.
/// An entry added or removed during the iteration may or may not be
/// returned, but no entry is returned twice. The iteration ends if the
/// directory is removed.
pub struct TestReadDir {
    shared: Arc<Shared>,
    dir: PathBuf,
    path: PathBuf,
    cursor: Option<(u64, OsString)>,
}

impl Iterator for TestReadDir {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        let tree = lock(&self.shared.tree);
        let Some(TestFsEntry::Dir(dir)) = tree.get(&self.dir) else {
            return None;
        };
        let after = self
            .cursor
            .as_ref()
            .map_or(Bound::Unbounded, Bound::Excluded);
        let (child, entry) = dir
            .children
            .range::<(u64, OsString), _>((after, Bound::Unbounded))
            .find_map(|child| {
                tree.get(&self.dir.join(&child.1))
                    .map(|entry| (child, entry_file_type(entry)))
            })?;
        self.cursor = Some(child.clone());
        let path = self.path.join(&child.1);

        if let Err(error) = self
            .shared
            .inject(FsOp::ReadDirEntry, &self.dir.join(&child.1))
        {
            return Some(Err(fs_error_at(&path)(error)));
        }

        Some(Ok(DirEntry::new(path, entry)))
    }
}

fn is_dir(tree: &Tree, path: &Path) -> bool {
    matches!(tree.get(path), Some(TestFsEntry::Dir(_)))
}

fn entry_file_type(entry: &TestFsEntry) -> FileType {
    match entry {
        TestFsEntry::Dir(_) => FileType::Dir,
        TestFsEntry::File(_) => FileType::File,
        TestFsEntry::Symlink(_) => FileType::Symlink,
    }
}

/// Returns the [`Metadata`] of the entry at the resolved `path`, reported
/// for the `normalized_path` it was resolved from.
fn entry_metadata(tree: &Tree, normalized_path: &Path, path: &Path) -> Result<Metadata> {
    let (file_type, len, modified, created, attributes) = match tree.get(path) {
        Some(TestFsEntry::File(file)) => {
            let file = lock_file_read(file);
            (
                FileType::File,
                file.len,
                file.modified,
                file.created,
                file.attributes,
            )
        }
        Some(TestFsEntry::Dir(dir)) => {
            (FileType::Dir, 0, dir.modified, dir.created, dir.attributes)
        }
        Some(TestFsEntry::Symlink(symlink)) => {
            #[cfg(unix)]
            let len = symlink.target.as_os_str().len() as u64;
            #[cfg(windows)]
            let len = 0;

            (
                FileType::Symlink,
                len,
                symlink.created,
                symlink.created,
                symlink.attributes,
            )
        }
        None => {
            return Err(fs_error!(
                missing_kind(tree, path) =>
                "Metadata could not be read, path not found: {}",
                normalized_path.display()
            ));
        }
    };

    let metadata = Metadata::new(
        normalized_path,
        file_type,
        len,
        modified,
        created,
        attributes,
    );

    #[cfg(windows)]
    let metadata = Metadata {
        symlink_dir: matches!(tree.get(path), Some(TestFsEntry::Symlink(symlink)) if symlink.dir),
        ..metadata
    };

    Ok(metadata)
}

/// Returns the error for the file at `path` that could not be acted on,
/// the `action` named in it, because it is borrowed as `state`, if it is.
fn check_not_borrowed(state: BorrowState, path: &Path, action: &str) -> Result {
    match state {
        BorrowState::Read(0) => Ok(()),
        BorrowState::Read(count) => Err(fs_error!(
            ResourceBusy;
            "File could not be {action}, it is already borrowed for reading {count} times: {}",
            path.display()
        )),
        BorrowState::Write => Err(fs_error!(
            ResourceBusy;
            "File could not be {action}, it is already borrowed for writing: {}",
            path.display()
        )),
    }
}

/// Fails if the parent directory of the resolved `path` does not exist, the
/// `normalized_path` used in the error after `what` could not be done.
fn check_parent(tree: &Tree, path: &Path, normalized_path: &Path, what: &str) -> Result {
    if path.parent().is_some_and(|parent| is_dir(tree, parent)) {
        return Ok(());
    }

    Err(fs_error!(
        missing_kind(tree, path) =>
        "{what}, parent directory does not exist: {}",
        normalized_path.display()
    ))
}

/// Returns the `attributes` of an existing file overwritten by a copy of a
/// file with the `source` attributes: its permissions are copied, its
/// owner is kept.
#[cfg(unix)]
fn overwritten_attributes(
    attributes: NativeAttributes,
    source: NativeAttributes,
) -> NativeAttributes {
    NativeAttributes {
        mode: source.mode,
        ..attributes
    }
}

/// Returns the attributes of an existing file overwritten by a copy of a
/// file with the `source` attributes, which are all copied.
#[cfg(windows)]
fn overwritten_attributes(_: NativeAttributes, source: NativeAttributes) -> NativeAttributes {
    copied_attributes(source)
}

/// Returns the attributes of a new copy of a file, which keeps only the
/// permissions on Unix and all the attributes on Windows, see
/// [`Fs::copy`].
fn copied_attributes(attributes: NativeAttributes) -> NativeAttributes {
    #[cfg(unix)]
    let copied = NativeAttributes {
        mode: attributes.mode,
        uid: OWNER_ID,
        gid: OWNER_ID,
    };
    #[cfg(windows)]
    let copied = attributes;

    copied
}

fn has_children(tree: &Tree, path: &Path) -> bool {
    matches!(tree.get(path), Some(TestFsEntry::Dir(dir)) if !dir.children.is_empty())
}

/// Inserts the `entry` at `path`, adds it to the children of its parent
/// directory and updates the modification time of the parent.
fn insert_entry(tree: &mut Tree, path: PathBuf, entry: TestFsEntry) {
    if let Some((parent, name)) = parent_dir(tree, &path) {
        parent
            .children
            .insert((name_hash(name), name.to_os_string()));
        parent.modified = SystemTime::now();
    }

    tree.insert(path, entry);
}

/// Removes the entry at `path`, removes it from the children of its parent
/// directory and updates the modification time of the parent.
fn remove_entry(tree: &mut Tree, path: &Path) -> Option<TestFsEntry> {
    if let Some((parent, name)) = parent_dir(tree, path) {
        parent
            .children
            .remove(&(name_hash(name), name.to_os_string()));
        parent.modified = SystemTime::now();
    }

    tree.remove(path)
}

/// Returns the parent directory of `path` and its name, if the parent is a
/// directory.
fn parent_dir<'a>(tree: &'a mut Tree, path: &'a Path) -> Option<(&'a mut TestFsDir, &'a OsStr)> {
    match path.parent().and_then(|parent| tree.get_mut(parent)) {
        Some(TestFsEntry::Dir(dir)) => path.file_name().map(|name| (dir, name)),
        _ => None,
    }
}

impl Fs for TestFs {
    type ReadHandle = TestFileRead;
    type WriteHandle = TestFileWrite;
    type Dir = TestReadDir;

    fn open_read<P: AsRef<Path>>(&self, path: P) -> Result<Self::ReadHandle> {
        let (tree, normalized_path, resolved_path) = self.lock_resolve(path.as_ref(), true)?;
        self.inject(FsOp::Open, &resolved_path, &normalized_path)?;

        match tree.get(&resolved_path) {
            Some(TestFsEntry::File(file)) => {
                match &mut lock_file_write(file).state {
                    BorrowState::Read(count) => *count += 1,
                    BorrowState::Write => {
                        return Err(fs_error!(
                            ResourceBusy;
                            "File could not be opened, it is already borrowed for writing: {}",
                            normalized_path.display()
                        ));
                    }
                }

                Ok(TestFileRead(self.handle(
                    file,
                    normalized_path,
                    resolved_path,
                )))
            }
            Some(TestFsEntry::Dir(_)) => Err(fs_error!(
                IsADirectory;
                "File could not be opened, path is a directory: {}",
                normalized_path.display()
            )),
            Some(TestFsEntry::Symlink(_)) | None => Err(fs_error!(
                missing_kind(&tree, &resolved_path) =>
                "File could not be opened, file not found: {}",
                normalized_path.display()
            )),
        }
    }

    fn open_write_options<P: AsRef<Path>>(
        &self,
        path: P,
        options: OpenWriteOptions,
    ) -> Result<Self::WriteHandle> {
        let (mut tree, normalized_path) = self.lock_normalize(path.as_ref())?;

        // WHY: like `std::fs::OpenOptions`, which rejects the combination
        // before opening anything.
        if options.append && options.truncate && !options.create_new {
            return Err(fs_error!(
                InvalidInput;
                "File could not be opened with both 'append' and 'truncate' options: {}",
                normalized_path.display()
            ));
        }

        let create = options.create || options.create_new;

        #[cfg(unix)]
        if create {
            self.check_current_dir(&tree, &normalized_path, "File could not be created")?;
        }

        // WHY: like `O_CREAT`, a path that must be a directory cannot be
        // created as a file, whether it exists or not.
        if create && requires_dir(&normalized_path) {
            let link_path = self.resolve(&tree, &normalized_path, false)?;
            check_parent(
                &tree,
                &link_path,
                &normalized_path,
                "File could not be created",
            )?;

            return Err(fs_error!(
                IsADirectory;
                "File could not be opened, path is a directory: {}",
                normalized_path.display()
            ));
        }

        let link_path = self.resolve_dir(&tree, &normalized_path, false)?;
        let resolved_path = self.resolve(&tree, &link_path, true)?;
        self.inject(FsOp::Open, &resolved_path, &normalized_path)?;

        // WHY: like `O_EXCL`, `create_new` does not follow a symlink, even
        // a dangling one.
        if options.create_new && tree.contains_key(&link_path) {
            return Err(fs_error!(
                AlreadyExists;
                "File could not be created, path already exists: {}",
                normalized_path.display()
            ));
        }

        let file = match tree.get(&resolved_path) {
            Some(TestFsEntry::File(file)) => {
                let mut inode = lock_file_write(file);

                if inode.attributes.attributes().readonly {
                    return Err(fs_error!(
                        PermissionDenied;
                        "File could not be opened, it is read-only: {}",
                        normalized_path.display()
                    ));
                }

                check_not_borrowed(inode.state, &normalized_path, "opened")?;
                inode.state = BorrowState::Write;

                if options.truncate {
                    inode.set_len(0);
                }

                file.clone()
            }
            Some(TestFsEntry::Dir(_)) => {
                return Err(fs_error!(
                    IsADirectory;
                    "File could not be opened, path is a directory: {}",
                    normalized_path.display()
                ));
            }
            Some(TestFsEntry::Symlink(_)) | None if !create => {
                return Err(fs_error!(
                    missing_kind(&tree, &resolved_path) =>
                    "File could not be opened, file not found: {}",
                    normalized_path.display()
                ));
            }
            Some(TestFsEntry::Symlink(_)) | None => {
                check_parent(
                    &tree,
                    &resolved_path,
                    &normalized_path,
                    "File could not be created",
                )?;

                let mut new_file =
                    TestFsFile::new(SystemTime::now(), new_file_attributes(&options));
                new_file.state = BorrowState::Write;
                let new_file = Arc::new(RwLock::new(new_file));
                insert_entry(
                    &mut tree,
                    resolved_path.clone(),
                    TestFsEntry::File(new_file.clone()),
                );
                new_file
            }
        };

        Ok(TestFileWrite {
            handle: self.handle(&file, normalized_path, resolved_path),
            append: options.append,
            write_only: options.write_only,
        })
    }

    fn read_dir<P: AsRef<Path>>(&self, path: P) -> Result<Self::Dir> {
        let (tree, normalized_path, resolved_path) = self.lock_resolve(path.as_ref(), true)?;
        self.inject(FsOp::ReadDir, &resolved_path, &normalized_path)?;

        match tree.get(&resolved_path) {
            Some(TestFsEntry::Dir(_)) => Ok(TestReadDir {
                shared: self.shared.clone(),
                dir: resolved_path,
                path: normalized_path,
                cursor: None,
            }),
            Some(TestFsEntry::File(_)) => Err(fs_error!(
                NotADirectory;
                "Directory could not be read, path is not a directory: {}",
                normalized_path.display()
            )),
            Some(TestFsEntry::Symlink(_)) | None => Err(fs_error!(
                missing_kind(&tree, &resolved_path) =>
                "Directory could not be read, path not found: {}",
                normalized_path.display()
            )),
        }
    }

    fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result {
        // WHY: like `mkdir`, a trailing separator neither follows a symlink
        // nor fails for an existing entry other than a directory.
        let (mut tree, normalized_path) = self.lock_normalize(path.as_ref())?;
        #[cfg(unix)]
        self.check_current_dir(&tree, &normalized_path, "Directory could not be created")?;
        let link_path = self.resolve(&tree, &normalized_path, false)?;
        self.inject(FsOp::CreateDir, &link_path, &normalized_path)?;

        if tree.contains_key(&link_path) {
            return Err(fs_error!(
                AlreadyExists;
                "Directory could not be created, path already exists: {}",
                normalized_path.display()
            ));
        }

        check_parent(
            &tree,
            &link_path,
            &normalized_path,
            "Directory could not be created",
        )?;
        insert_entry(
            &mut tree,
            link_path,
            TestFsEntry::Dir(TestFsDir::new(SystemTime::now())),
        );

        Ok(())
    }

    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result {
        let (mut tree, normalized_path) = self.lock_normalize(path.as_ref())?;
        // WHY: a `..` after a missing parent resolves only once it is
        // created, until then faults match the path as given.
        let resolved = self
            .resolve(&tree, &normalized_path, false)
            .unwrap_or_else(|_| normalized_path.components().collect());
        self.inject(FsOp::CreateDir, &resolved, &normalized_path)?;
        // WHY: an empty path is not a directory to create, as with
        // `std::fs::create_dir_all`.
        let ancestors: Vec<&Path> = normalized_path
            .ancestors()
            .filter(|ancestor| !ancestor.as_os_str().is_empty())
            .collect();
        let mut parent = PathBuf::new();

        // WHY: each ancestor is created like `std::fs::create_dir_all` does,
        // which follows a symlink to a directory but does not create the
        // target of a dangling one.
        for ancestor in ancestors.into_iter().rev() {
            let link = ancestor
                .file_name()
                .map_or_else(|| ancestor.to_path_buf(), |name| parent.join(name));
            let dir = self.resolve(&tree, &link, true)?;

            match tree.get(&dir) {
                Some(TestFsEntry::Dir(_)) => {}
                // WHY: as with `mkdir`, a file fails as an existing path,
                // a file in a parent component as not a directory.
                Some(TestFsEntry::File(_)) => {
                    let kind = if ancestor == normalized_path {
                        std::io::ErrorKind::AlreadyExists
                    } else {
                        std::io::ErrorKind::NotADirectory
                    };

                    return Err(fs_error!(
                        kind =>
                        "Directory could not be created, a file already exists at: {}",
                        link.display()
                    ));
                }
                Some(TestFsEntry::Symlink(_)) | None => {
                    if matches!(tree.get(&link), Some(TestFsEntry::Symlink(_))) {
                        return Err(fs_error!(
                            AlreadyExists;
                            "Directory could not be created, a dangling symlink exists at: {}",
                            link.display()
                        ));
                    }

                    insert_entry(
                        &mut tree,
                        dir.clone(),
                        TestFsEntry::Dir(TestFsDir::new(SystemTime::now())),
                    );
                }
            }

            parent = dir;
        }

        Ok(())
    }

    fn exists<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        let (tree, normalized_path, resolved_path) = self.lock_resolve(path.as_ref(), true)?;
        self.inject(FsOp::Metadata, &resolved_path, &normalized_path)?;

        if tree.contains_key(&resolved_path) {
            return Ok(true);
        }

        // WHY: Windows reports a file as a parent as not found.
        #[cfg(unix)]
        if missing_kind(&tree, &resolved_path) == std::io::ErrorKind::NotADirectory {
            return Err(fs_error!(
                NotADirectory;
                "Existence could not be checked, not a directory: {}",
                normalized_path.display()
            ));
        }

        Ok(false)
    }

    fn metadata<P: AsRef<Path>>(&self, path: P) -> Result<Metadata> {
        let (tree, normalized_path, resolved_path) = self.lock_resolve(path.as_ref(), true)?;
        self.inject(FsOp::Metadata, &resolved_path, &normalized_path)?;
        entry_metadata(&tree, &normalized_path, &resolved_path)
    }

    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> Result<Metadata> {
        let (tree, normalized_path, link_path) = self.lock_resolve(path.as_ref(), false)?;
        self.inject(FsOp::Metadata, &link_path, &normalized_path)?;
        entry_metadata(&tree, &normalized_path, &link_path)
    }

    fn create_symlink_file<P: AsRef<Path>, T: AsRef<Path>>(&self, path: P, target: T) -> Result {
        self.add_symlink(path.as_ref(), target.as_ref(), false)
    }

    fn create_symlink_dir<P: AsRef<Path>, T: AsRef<Path>>(&self, path: P, target: T) -> Result {
        self.add_symlink(path.as_ref(), target.as_ref(), true)
    }

    fn read_link<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let (tree, normalized_path, link_path) = self.lock_resolve(path.as_ref(), false)?;
        self.inject(FsOp::ReadLink, &link_path, &normalized_path)?;

        match tree.get(&link_path) {
            Some(TestFsEntry::Symlink(symlink)) => Ok(symlink.target.clone()),
            Some(TestFsEntry::File(_)) | Some(TestFsEntry::Dir(_)) => Err(fs_error!(
                "Symlink could not be read, path is not a symlink: {}",
                normalized_path.display()
            )
            .with_source(not_symlink_error())),
            None => Err(fs_error!(
                missing_kind(&tree, &link_path) =>
                "Symlink could not be read, path not found: {}",
                normalized_path.display()
            )),
        }
    }

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf> {
        let (tree, normalized_path, resolved_path) = self.lock_resolve(path.as_ref(), true)?;
        self.inject(FsOp::Canonicalize, &resolved_path, &normalized_path)?;

        if tree.contains_key(&resolved_path) {
            return Ok(resolved_path);
        }

        Err(fs_error!(
            missing_kind(&tree, &resolved_path) =>
            "Path could not be canonicalized, path not found: {}",
            normalized_path.display()
        ))
    }

    fn create_hard_link<P: AsRef<Path>, T: AsRef<Path>>(&self, path: P, target: T) -> Result {
        let current_dir = self.env.current_dir_arc();
        let normalized_target = normalize_at(&current_dir, target.as_ref())?;
        let (mut tree, normalized_path) = self.lock_normalize_at(&current_dir, path.as_ref())?;
        let link_path = self.resolve(&tree, &normalized_path, false)?;
        self.inject(FsOp::CreateLink, &link_path, &normalized_path)?;

        if tree.contains_key(&link_path) {
            return Err(fs_error!(
                AlreadyExists;
                "Hard link could not be created, path already exists: {}",
                normalized_path.display()
            ));
        }

        check_parent(
            &tree,
            &link_path,
            &normalized_path,
            "Hard link could not be created",
        )?;

        // WHY: like `link`, which cannot create a directory.
        if requires_dir(&normalized_path) {
            return Err(fs_error!(
                NotFound;
                "Hard link could not be created, path ends with a separator: {}",
                normalized_path.display()
            ));
        }

        let target_path = self.resolve_dir(&tree, &normalized_target, false)?;
        let entry = match tree.get(&target_path) {
            Some(TestFsEntry::File(file)) => TestFsEntry::File(file.clone()),
            Some(TestFsEntry::Symlink(symlink)) => {
                self.symlinks.fetch_add(1, Ordering::Relaxed);
                TestFsEntry::Symlink(symlink.clone())
            }
            Some(TestFsEntry::Dir(_)) => {
                return Err(fs_error!(
                    PermissionDenied;
                    "Hard link could not be created, target is a directory: {}",
                    normalized_target.display()
                ));
            }
            None => {
                return Err(fs_error!(
                    missing_kind(&tree, &target_path) =>
                    "Hard link could not be created, target not found: {}",
                    normalized_target.display()
                ));
            }
        };
        insert_entry(&mut tree, link_path, entry);

        Ok(())
    }

    fn copy<S: AsRef<Path>, T: AsRef<Path>>(&self, source: S, target: T) -> Result<u64> {
        let current_dir = self.env.current_dir_arc();
        let normalized_source = normalize_at(&current_dir, source.as_ref())?;
        let (mut tree, normalized_target) =
            self.lock_normalize_at(&current_dir, target.as_ref())?;
        let source_path = self.resolve_dir(&tree, &normalized_source, true)?;
        self.inject(FsOp::Open, &source_path, &normalized_source)?;

        let source = match tree.get(&source_path) {
            Some(TestFsEntry::File(file)) => file.clone(),
            Some(TestFsEntry::Dir(_)) => {
                // WHY: std refuses a directory on Unix, Windows denies
                // opening it.
                #[cfg(unix)]
                const NOT_FILE: std::io::ErrorKind = std::io::ErrorKind::InvalidInput;
                #[cfg(windows)]
                const NOT_FILE: std::io::ErrorKind = std::io::ErrorKind::PermissionDenied;

                return Err(fs_error!(
                    NOT_FILE =>
                    "File could not be copied, source is not a file: {}",
                    normalized_source.display()
                ));
            }
            Some(TestFsEntry::Symlink(_)) | None => {
                return Err(fs_error!(
                    missing_kind(&tree, &source_path) =>
                    "File could not be copied, source not found: {}",
                    normalized_source.display()
                ));
            }
        };
        let (data, len, attributes, source_state) = {
            let source = lock_file_read(&source);
            (
                source.data.clone(),
                source.len,
                source.attributes,
                source.state,
            )
        };

        if source_state == BorrowState::Write {
            return Err(fs_error!(
                ResourceBusy;
                "File could not be opened, it is already borrowed for writing: {}",
                normalized_source.display()
            ));
        }

        // WHY: like `O_CREAT`, a path that must be a directory cannot be
        // created as a file, whether it exists or not.
        if requires_dir(&normalized_target) {
            let link_path = self.resolve(&tree, &normalized_target, false)?;
            check_parent(
                &tree,
                &link_path,
                &normalized_target,
                "File could not be copied",
            )?;

            return Err(fs_error!(
                IsADirectory;
                "File could not be copied, target is a directory: {}",
                normalized_target.display()
            ));
        }

        let target_path = self.resolve(&tree, &normalized_target, true)?;
        self.inject(FsOp::Open, &target_path, &normalized_target)?;
        let target = match tree.get(&target_path) {
            Some(TestFsEntry::Dir(_)) => {
                return Err(fs_error!(
                    IsADirectory;
                    "File could not be copied, target is a directory: {}",
                    normalized_target.display()
                ));
            }
            Some(TestFsEntry::File(target)) => {
                let target_file = lock_file_read(target);
                // WHY: the source is open for reading while it is copied, so
                // as the target it is borrowed once more.
                let state = match target_file.state {
                    BorrowState::Read(count) if Arc::ptr_eq(target, &source) => {
                        BorrowState::Read(count + 1)
                    }
                    state => state,
                };
                check_not_borrowed(state, &normalized_target, "opened")?;

                if target_file.attributes.attributes().readonly {
                    return Err(fs_error!(
                        PermissionDenied;
                        "File could not be opened, it is read-only: {}",
                        normalized_target.display()
                    ));
                }

                // WHY: `CopyFileExW` recreates the target with the source
                // attributes, which Windows denies for a hidden file unless
                // they keep it hidden.
                #[cfg(windows)]
                if target_file.attributes.hidden && !attributes.hidden {
                    return Err(fs_error!(
                        PermissionDenied;
                        "File could not be opened, it is hidden: {}",
                        normalized_target.display()
                    ));
                }

                Some((target.clone(), target_file.len))
            }
            Some(TestFsEntry::Symlink(_)) | None => {
                check_parent(
                    &tree,
                    &target_path,
                    &normalized_target,
                    "File could not be copied",
                )?;
                None
            }
        };

        self.inject(FsOp::Read, &source_path, &normalized_source)?;
        self.inject(FsOp::Write, &target_path, &normalized_target)?;

        if let Some(capacity) = self.shared.capacity() {
            let (file, current_len) = target
                .as_ref()
                .map_or((None, 0), |(file, len)| (Some(file), *len));
            check_capacity(&tree, capacity, file, current_len, len)
                .map_err(fs_error_at(&normalized_target))?;
        }

        match target {
            Some((target, _)) => {
                let mut target = lock_file_write(&target);
                target.data = data;
                target.len = len;
                target.modified = SystemTime::now();
                target.attributes = overwritten_attributes(target.attributes, attributes);
            }
            None => {
                let mut copy = TestFsFile::new(SystemTime::now(), copied_attributes(attributes));
                copy.data = data;
                copy.len = len;
                insert_entry(
                    &mut tree,
                    target_path,
                    TestFsEntry::File(Arc::new(RwLock::new(copy))),
                );
            }
        }

        Ok(len)
    }

    fn copy_dir_all<S: AsRef<Path>, T: AsRef<Path>>(&self, source: S, target: T) -> Result {
        let current_dir = self.env.current_dir_arc();
        copy_tree(
            self,
            &normalize_at(&current_dir, source.as_ref())?,
            &normalize_at(&current_dir, target.as_ref())?,
        )
    }

    fn remove_file<P: AsRef<Path>>(&self, path: P) -> Result {
        let (mut tree, normalized_path, link_path) = self.lock_resolve(path.as_ref(), false)?;
        self.inject(FsOp::Remove, &link_path, &normalized_path)?;
        #[cfg(unix)]
        self.check_not_dir_symlink(
            &tree,
            &normalized_path,
            &link_path,
            "File could not be removed",
        )?;

        match tree.get(&link_path) {
            Some(TestFsEntry::File(file)) => {
                check_not_borrowed(lock_file_read(file).state, &normalized_path, "removed")?;
            }
            Some(TestFsEntry::Symlink(_)) => {
                self.symlinks.fetch_sub(1, Ordering::Relaxed);
            }
            Some(TestFsEntry::Dir(_)) => {
                return Err(fs_error!(
                    IsADirectory;
                    "File could not be removed, path is a directory: {}",
                    normalized_path.display()
                ));
            }
            None => {
                return Err(fs_error!(
                    missing_kind(&tree, &link_path) =>
                    "File could not be removed, path not found: {}",
                    normalized_path.display()
                ));
            }
        }

        remove_entry(&mut tree, &link_path);

        Ok(())
    }

    fn remove_dir<P: AsRef<Path>>(&self, path: P) -> Result {
        let (mut tree, normalized_path, link_path) = self.lock_resolve(path.as_ref(), false)?;
        self.inject(FsOp::Remove, &link_path, &normalized_path)?;
        #[cfg(unix)]
        self.check_not_dir_symlink(
            &tree,
            &normalized_path,
            &link_path,
            "Directory could not be removed",
        )?;

        match tree.get(&link_path) {
            Some(TestFsEntry::Dir(_)) if link_path.parent().is_none() => {
                return Err(fs_error!(
                    ResourceBusy;
                    "Directory could not be removed, path is a root: {}",
                    normalized_path.display()
                ));
            }
            Some(TestFsEntry::Dir(_)) if has_children(&tree, &link_path) => {
                return Err(fs_error!(
                    DirectoryNotEmpty;
                    "Directory could not be removed, it is not empty: {}",
                    normalized_path.display()
                ));
            }
            Some(TestFsEntry::Dir(_)) => {}
            // WHY: on Windows a directory symlink is a directory entry
            // removed like a directory.
            #[cfg(windows)]
            Some(TestFsEntry::Symlink(symlink)) if symlink.dir => {
                self.symlinks.fetch_sub(1, Ordering::Relaxed);
            }
            Some(TestFsEntry::File(_)) | Some(TestFsEntry::Symlink(_)) => {
                return Err(fs_error!(
                    NotADirectory;
                    "Directory could not be removed, path is not a directory: {}",
                    normalized_path.display()
                ));
            }
            None => {
                return Err(fs_error!(
                    missing_kind(&tree, &link_path) =>
                    "Directory could not be removed, path not found: {}",
                    normalized_path.display()
                ));
            }
        }

        remove_entry(&mut tree, &link_path);

        Ok(())
    }

    fn remove_dir_all<P: AsRef<Path>>(&self, path: P) -> Result {
        let (mut tree, normalized_path, link_path) = self.lock_resolve(path.as_ref(), false)?;
        self.inject(FsOp::Remove, &link_path, &normalized_path)?;
        check_not_root(&link_path)?;
        self.check_not_dir_symlink(
            &tree,
            &normalized_path,
            &link_path,
            "Directory could not be removed",
        )?;

        match tree.get(&link_path) {
            Some(TestFsEntry::Dir(_)) | Some(TestFsEntry::Symlink(_)) => {}
            Some(TestFsEntry::File(_)) => {
                return Err(fs_error!(
                    NotADirectory;
                    "Directory could not be removed, path is not a directory: {}",
                    normalized_path.display()
                ));
            }
            None => {
                return Err(fs_error!(
                    missing_kind(&tree, &link_path) =>
                    "Directory could not be removed, path not found: {}",
                    normalized_path.display()
                ));
            }
        }

        // WHY: a symlink has no descendants, so only the symlink itself is
        // removed.
        let removed: Vec<PathBuf> = tree
            .range::<Path, _>((Bound::Included(link_path.as_path()), Bound::Unbounded))
            .take_while(|(path, _)| path.starts_with(&link_path))
            .map(|(path, _)| path.clone())
            .collect();

        for path in &removed {
            if let Some(TestFsEntry::File(file)) = tree.get(path) {
                let display_path =
                    normalized_path.join(path.strip_prefix(&link_path).unwrap_or(path));
                check_not_borrowed(lock_file_read(file).state, &display_path, "removed")?;
            }
        }

        for path in removed.iter().rev() {
            if let Some(TestFsEntry::Symlink(_)) = remove_entry(&mut tree, path) {
                self.symlinks.fetch_sub(1, Ordering::Relaxed);
            }
        }

        Ok(())
    }

    fn rename<S: AsRef<Path>, T: AsRef<Path>>(&self, source: S, target: T) -> Result {
        let current_dir = self.env.current_dir_arc();
        let normalized_source = normalize_at(&current_dir, source.as_ref())?;
        let (mut tree, normalized_target) =
            self.lock_normalize_at(&current_dir, target.as_ref())?;
        let target_path = self.resolve(&tree, &normalized_target, false)?;
        let source_path = self.resolve(&tree, &normalized_source, false)?;
        self.inject(FsOp::Rename, &source_path, &normalized_source)?;

        let Some(source_entry) = tree.get(&source_path) else {
            return Err(fs_error!(
                missing_kind(&tree, &source_path) =>
                "Path could not be renamed, source not found: {}",
                normalized_source.display()
            ));
        };

        // WHY: like `rename`, a trailing separator does not follow a
        // symlink but requires a directory.
        if requires_dir(&normalized_source) && !matches!(source_entry, TestFsEntry::Dir(_)) {
            return Err(fs_error!(
                NotADirectory;
                "Path could not be renamed, source is not a directory: {}",
                normalized_source.display()
            ));
        }

        if source_path == target_path {
            return self.check_rename_target_dir(
                &tree,
                source_entry,
                &normalized_target,
                &target_path,
            );
        }

        if source_path.parent().is_none() {
            return Err(fs_error!(
                ResourceBusy;
                "Path could not be renamed, source is a root: {}",
                normalized_source.display()
            ));
        }

        check_parent(
            &tree,
            &target_path,
            &normalized_target,
            "Path could not be renamed",
        )?;
        self.check_rename_target_dir(&tree, source_entry, &normalized_target, &target_path)?;

        if target_path.starts_with(&source_path) {
            return Err(fs_error!(
                InvalidInput;
                "Directory could not be renamed, target is inside source: {}",
                normalized_target.display()
            ));
        }

        // WHY: the rules of POSIX `rename`, which `std::fs::rename` follows
        // on every platform.
        match (source_entry, tree.get(&target_path)) {
            (_, None) => {}
            (TestFsEntry::File(source_file), Some(TestFsEntry::File(target_file)))
                if Arc::ptr_eq(source_file, target_file) =>
            {
                // WHY: POSIX `rename` does nothing for two links of the same
                // file, Windows removes the source.
                #[cfg(windows)]
                remove_entry(&mut tree, &source_path);

                return Ok(());
            }
            (TestFsEntry::Dir(_), Some(TestFsEntry::Dir(_))) => {
                if has_children(&tree, &target_path) {
                    return Err(fs_error!(
                        DirectoryNotEmpty;
                        "Directory could not be renamed, target is not empty: {}",
                        normalized_target.display()
                    ));
                }
            }
            (TestFsEntry::Dir(_), Some(_)) => {
                return Err(fs_error!(
                    NotADirectory;
                    "Directory could not be renamed, target is not a directory: {}",
                    normalized_target.display()
                ));
            }
            (_, Some(TestFsEntry::Dir(_))) => {
                return Err(fs_error!(
                    IsADirectory;
                    "Path could not be renamed, target is a directory: {}",
                    normalized_target.display()
                ));
            }
            (_, Some(TestFsEntry::File(target_file))) => {
                check_not_borrowed(
                    lock_file_read(target_file).state,
                    &normalized_target,
                    "replaced",
                )?;
            }
            (_, Some(TestFsEntry::Symlink(_))) => {}
        }

        if let Some(TestFsEntry::Symlink(_)) = remove_entry(&mut tree, &target_path) {
            self.symlinks.fetch_sub(1, Ordering::Relaxed);
        }

        let moved: Vec<PathBuf> = tree
            .range::<Path, _>((Bound::Included(source_path.as_path()), Bound::Unbounded))
            .take_while(|(path, _)| path.starts_with(&source_path))
            .map(|(path, _)| path.clone())
            .collect();
        let entries: Vec<(PathBuf, TestFsEntry)> = moved
            .into_iter()
            .filter_map(|path| remove_entry(&mut tree, &path).map(|entry| (path, entry)))
            .collect();

        for (path, entry) in entries {
            let moved = target_path.join(path.strip_prefix(&source_path).unwrap_or(&path));
            insert_entry(&mut tree, moved, entry);
        }

        Ok(())
    }

    fn set_readonly<P: AsRef<Path>>(&self, path: P, readonly: bool) -> Result {
        self.set_permissions(path.as_ref(), |attributes| {
            #[cfg(unix)]
            {
                attributes.mode = readonly_mode(attributes.mode, readonly);
            }
            #[cfg(windows)]
            {
                attributes.readonly = readonly;
            }
        })
    }

    #[cfg(unix)]
    fn set_executable<P: AsRef<Path>>(&self, path: P, executable: bool) -> Result {
        let (tree, normalized_path, resolved_path) = self.lock_resolve(path.as_ref(), true)?;

        if is_dir(&tree, &resolved_path) {
            self.inject(FsOp::SetPermissions, &resolved_path, &normalized_path)?;

            return Err(fs_error!(
                IsADirectory;
                "Executable attribute could not be set, path is a directory: {}",
                normalized_path.display()
            ));
        }

        drop(tree);
        self.set_permissions(path.as_ref(), |attributes| {
            attributes.mode = executable_mode(attributes.mode, executable);
        })
    }

    fn set_modified<P: AsRef<Path>>(&self, path: P, modified: SystemTime) -> Result {
        let (mut tree, normalized_path, resolved_path) = self.lock_resolve(path.as_ref(), true)?;
        self.inject(FsOp::SetTimes, &resolved_path, &normalized_path)?;

        match tree.get_mut(&resolved_path) {
            Some(TestFsEntry::File(file)) => lock_file_write(file).modified = modified,
            Some(TestFsEntry::Dir(dir)) => dir.modified = modified,
            Some(TestFsEntry::Symlink(_)) | None => {
                return Err(fs_error!(
                    missing_kind(&tree, &resolved_path) =>
                    "Modified time could not be set, path not found: {}",
                    normalized_path.display()
                ));
            }
        }

        Ok(())
    }

    #[cfg(unix)]
    fn set_mode<P: AsRef<Path>>(&self, path: P, mode: u32) -> Result {
        self.set_permissions(path.as_ref(), |attributes| {
            attributes.mode = mode & 0o7777;
        })
    }
}

impl TestFs {
    /// Adds a symlink at `path` to `target`, a directory symlink if `dir`,
    /// see [`Fs::create_symlink_file`].
    fn add_symlink(&self, path: &Path, target: &Path, dir: bool) -> Result {
        let (mut tree, normalized_path) = self.lock_normalize(path)?;
        let link_path = self.resolve(&tree, &normalized_path, false)?;
        self.inject(FsOp::CreateLink, &link_path, &normalized_path)?;

        #[cfg(unix)]
        let target = target.to_path_buf();
        #[cfg(windows)]
        let target = windows_symlink_target(target);

        if tree.contains_key(&link_path) {
            return Err(fs_error!(
                AlreadyExists;
                "Symlink could not be created, path already exists: {}",
                normalized_path.display()
            ));
        }

        check_parent(
            &tree,
            &link_path,
            &normalized_path,
            "Symlink could not be created",
        )?;

        // WHY: like `symlink`, which cannot create a directory.
        if requires_dir(&normalized_path) {
            return Err(fs_error!(
                NotFound;
                "Symlink could not be created, path ends with a separator: {}",
                normalized_path.display()
            ));
        }

        // WHY: `Path::components` ignores an empty target, which the OS
        // rejects.
        if target.as_os_str().is_empty() {
            return Err(fs_error!(
                NotFound;
                "Symlink could not be created, target is empty: {}",
                normalized_path.display()
            ));
        }

        insert_entry(
            &mut tree,
            link_path,
            TestFsEntry::Symlink(TestFsSymlink::new(target, dir, SystemTime::now())),
        );
        self.symlinks.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Sets the permissions of the entry at `path`, followed if it is a
    /// symlink, with `update`.
    fn set_permissions(&self, path: &Path, update: impl FnOnce(&mut NativeAttributes)) -> Result {
        let (tree, normalized_path, resolved_path) = self.lock_resolve(path, true)?;
        self.inject(FsOp::SetPermissions, &resolved_path, &normalized_path)?;
        drop(tree);
        self.update_attributes(path, update)
    }

    /// Fails if the normalized `path` [`requires_dir`] but is a symlink, to
    /// the directory at `dir`, which the OS follows but does not remove, e.g.
    /// [`Fs::remove_dir_all`] only after removing the content.
    fn check_not_dir_symlink(&self, tree: &Tree, path: &Path, dir: &Path, what: &str) -> Result {
        if requires_dir(path)
            && tree.contains_key(dir)
            && self
                .resolve(tree, path, false)
                .is_ok_and(|link| matches!(tree.get(&link), Some(TestFsEntry::Symlink(_))))
        {
            return Err(fs_error!(
                NotADirectory;
                "{what}, path is a symlink: {}",
                path.display()
            ));
        }

        Ok(())
    }

    /// Fails if the normalized `target` of [`Fs::rename`], at the
    /// unresolved `target_path`, [`requires_dir`] but the `source` cannot
    /// replace it as a directory, like `rename` which does not follow it.
    fn check_rename_target_dir(
        &self,
        tree: &Tree,
        source: &TestFsEntry,
        target: &Path,
        target_path: &Path,
    ) -> Result {
        if !requires_dir(target) {
            return Ok(());
        }

        match (source, tree.get(target_path)) {
            (TestFsEntry::Dir(_), None | Some(TestFsEntry::Dir(_))) => Ok(()),
            (TestFsEntry::Dir(_), Some(_)) => Err(fs_error!(
                NotADirectory;
                "Directory could not be renamed, target is not a directory: {}",
                target.display()
            )),
            _ => Err(fs_error!(
                NotADirectory;
                "Path could not be renamed, target is not a directory: {}",
                target.display()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;
    use std::io::IoSlice;

    use super::*;

    /// Returns the kind of the [`std::io::Error`] that caused the failure
    /// of `result`.
    #[track_caller]
    fn kind<T>(result: Result<T>) -> ErrorKind {
        result
            .err()
            .expect("should fail")
            .source_as::<std::io::Error>()
            .expect("should be caused by an I/O error")
            .kind()
    }

    /// Returns the description of the failure of `result` with `/` as
    /// the separator, see [`shown`].
    #[track_caller]
    fn description<T>(result: Result<T>) -> String {
        let error = result.err().expect("should fail");
        error.description().replace('\\', "/")
    }

    /// Returns `path` resolved against the [`ROOT`] like by a new
    /// [`TestFs`], e.g. `C:/a.txt` for `/a.txt` on Windows.
    fn root(path: impl AsRef<Path>) -> PathBuf {
        Path::new(ROOT).join(path)
    }

    /// Returns `path` resolved against the [`ROOT`] as shown by
    /// [`description`].
    ///
    /// WHY: on Windows descriptions show a resolved path with `\` and
    /// an unresolved one with the separators as given.
    fn shown(path: impl AsRef<Path>) -> String {
        root(path).display().to_string().replace('\\', "/")
    }

    /// Returns a filesystem with the symlink `/loop` to itself, which no
    /// path through it resolves.
    fn looped() -> TestFs {
        let fs = TestFs::new();
        fs.create_symlink_file("/loop", "/loop").unwrap();
        fs
    }

    /// Returns the symlink at `path` as stored in the tree of `fs`, if any.
    fn symlink(fs: &TestFs, path: &str) -> Option<TestFsSymlink> {
        match lock(&fs.shared.tree).get(&root(path)) {
            Some(TestFsEntry::Symlink(symlink)) => Some(symlink.clone()),
            _ => None,
        }
    }

    /// Returns the borrow state of the file at `path` of `fs`, if any.
    fn state(fs: &TestFs, path: &str) -> Option<BorrowState> {
        match lock(&fs.shared.tree).get(&root(path)) {
            Some(TestFsEntry::File(file)) => Some(lock_file_read(file).state),
            _ => None,
        }
    }

    mod faults {
        use super::*;

        mod take {
            use super::*;

            #[test]
            fn take_should_return_last_result_repeatedly() {
                let mut faults = Faults::default();
                faults.rules.push(Rule {
                    op: FsOp::Open,
                    path: PathBuf::from("/a"),
                    fault: Fault::Errors(VecDeque::from([None, Some(ErrorKind::Other)])),
                });

                assert!(faults.take(FsOp::Open, Path::new("/a/b")).is_ok());
                assert!(faults.take(FsOp::Open, Path::new("/a/b")).is_err());
                assert!(faults.take(FsOp::Open, Path::new("/a/b")).is_err());
                assert!(faults.take(FsOp::Open, Path::new("/ab")).is_ok());
                assert!(faults.take(FsOp::Read, Path::new("/a")).is_ok());
            }

            #[test]
            fn take_should_name_operation_in_error() {
                let mut faults = Faults::default();
                faults.rules.push(Rule {
                    op: FsOp::SyncData,
                    path: PathBuf::new(),
                    fault: Fault::Errors(VecDeque::from([Some(ErrorKind::Other)])),
                });

                let error = faults.take(FsOp::SyncData, Path::new("/a")).unwrap_err();

                assert_eq!(error.to_string(), "simulated SyncData failure");
            }
        }
    }

    mod test_fs_file {
        use super::*;

        mod set_len {
            use super::*;

            #[test]
            fn set_len_unresolvable_path_should_fail() {
                let fs = looped();

                assert_eq!(kind(fs.set_len("", 0)), ErrorKind::NotFound);
            }

            #[test]
            fn set_len_should_cut_held_data() {
                let mut file = TestFsFile::new(SystemTime::UNIX_EPOCH, new_dir_attributes());
                file.data = b"hello".to_vec();
                file.len = 5;

                file.set_len(2);

                assert_eq!(file.data, b"he");
                assert!(file.modified > SystemTime::UNIX_EPOCH);
            }
        }
    }

    mod name_hash {
        use super::*;

        #[test]
        fn name_hash_should_be_fnv1a() {
            assert_eq!(name_hash(OsStr::new("")), 0xcbf2_9ce4_8422_2325);
            assert_eq!(name_hash(OsStr::new("a")), 0xaf63_dc4c_8601_ec8c);
        }
    }

    mod check_capacity {
        use super::*;

        #[test]
        fn check_capacity_should_count_hard_linked_file_once() {
            // GIVEN: a file of 4 bytes with two names
            let fs = TestFs::new();
            fs.write("/a.txt", "aaaa").unwrap();
            fs.create_hard_link("/b.txt", "/a.txt").unwrap();
            let tree = lock(&fs.shared.tree);

            // THEN: a new file of 4 bytes fits into 8 bytes
            assert!(check_capacity(&tree, 8, None, 0, 4).is_ok());
            assert_eq!(
                check_capacity(&tree, 7, None, 0, 4).unwrap_err().kind(),
                ErrorKind::StorageFull
            );
        }

        #[test]
        fn check_capacity_should_allow_shrinking_beyond_capacity() {
            let fs = TestFs::new();
            fs.write("/a.txt", "aaaa").unwrap();
            let tree = lock(&fs.shared.tree);

            assert!(check_capacity(&tree, 0, None, 4, 2).is_ok());
        }
    }

    mod lock {
        use super::*;

        #[test]
        fn lock_poisoned_mutex_should_return_value() {
            let mutex = Mutex::new(1);
            std::thread::scope(|scope| {
                scope
                    .spawn(|| {
                        let _guard = mutex.lock().unwrap();
                        panic!("poison");
                    })
                    .join()
                    .unwrap_err();
            });

            assert_eq!(*lock(&mutex), 1);
            assert!(!mutex.is_poisoned());
        }
    }

    mod test_fs {
        use super::*;

        #[test]
        fn test_fs_should_enforce_one_writer_or_many_readers() {
            // GIVEN: two read handles are open
            let fs = TestFs::new();
            fs.write("/a.txt", "aaa").unwrap();
            let read1 = fs.open_read("/a.txt").unwrap();
            let read2 = fs.open_read("/a.txt").unwrap();

            // THEN: writing fails as the file is borrowed for reading
            assert_eq!(
                description(fs.open_write("/a.txt")),
                format!(
                    "File could not be opened, it is already borrowed for reading 2 times: {}",
                    shown("/a.txt")
                )
            );

            // WHEN: the reads are dropped
            drop(read1);
            drop(read2);

            // THEN: the file can be opened for writing
            let write = fs.open_write("/a.txt").unwrap();

            // AND: neither reading nor another writer is allowed then
            assert_eq!(
                description(fs.open_read("/a.txt")),
                format!(
                    "File could not be opened, it is already borrowed for writing: {}",
                    shown("/a.txt")
                )
            );
            assert_eq!(kind(fs.open_write("/a.txt")), ErrorKind::ResourceBusy);
            assert_eq!(kind(fs.read("/a.txt")), ErrorKind::ResourceBusy);

            // WHEN: the writer is dropped
            drop(write);

            // THEN: the file can be read again
            assert_eq!(fs.read_to_string("/a.txt").unwrap(), "aaa");
        }

        #[test]
        fn test_fs_should_borrow_hard_linked_file_once() {
            let fs = TestFs::new();
            fs.write("/a.txt", "aaa").unwrap();
            fs.create_hard_link("/b.txt", "/a.txt").unwrap();
            let _read = fs.open_read("/a.txt").unwrap();

            assert_eq!(kind(fs.open_write("/b.txt")), ErrorKind::ResourceBusy);
        }

        #[test]
        fn test_fs_should_borrow_symlinked_file_under_its_target() {
            let fs = TestFs::new();
            fs.write("/a.txt", "aaa").unwrap();
            fs.create_symlink_file("/link", "a.txt").unwrap();
            let _write = fs.open_write("/link").unwrap();

            assert_eq!(kind(fs.open_read("/a.txt")), ErrorKind::ResourceBusy);
        }

        #[cfg(unix)]
        #[test]
        fn test_fs_should_resolve_parent_dir_physically() {
            // GIVEN: `/link` points to `/a/b`, whose parent is `/a`
            let fs = TestFs::new();
            fs.create_dir_all("/a/b").unwrap();
            fs.write("/a/x.txt", "x").unwrap();
            fs.create_symlink_dir("/link", "/a/b").unwrap();

            // THEN: `..` leaves the directory `/link` resolves to
            assert_eq!(fs.read_to_string("/link/../x.txt").unwrap(), "x");
            // AND: `..` fails through a missing or non-directory component
            assert_eq!(kind(fs.read("/missing/../a/x.txt")), ErrorKind::NotFound);
            assert_eq!(kind(fs.read("/a/x.txt/../x.txt")), ErrorKind::NotADirectory);
            assert_eq!(
                kind(fs.read("/a/x.txt/missing/../x")),
                ErrorKind::NotADirectory
            );
        }

        #[cfg(windows)]
        #[test]
        fn test_fs_should_remove_parent_dir_lexically() {
            // GIVEN: `/link` points to `/a/b`, whose parent is `/a`
            let fs = TestFs::new();
            fs.create_dir_all("/a/b").unwrap();
            fs.write("/a/x.txt", "x").unwrap();
            fs.write("/x.txt", "root").unwrap();
            fs.create_symlink_dir("/link", "/a/b").unwrap();

            // THEN: `..` removes `/link` itself before it is resolved
            assert_eq!(fs.read_to_string("/link/../x.txt").unwrap(), "root");
            // AND: `..` passes missing and non-directory components
            assert_eq!(fs.read_to_string("/missing/../a/x.txt").unwrap(), "x");
            assert_eq!(fs.read_to_string("/a/x.txt/../x.txt").unwrap(), "x");
            assert_eq!(
                fs.read_to_string("/a/x.txt/missing/../../x.txt").unwrap(),
                "x"
            );
            // AND: in a symlink target too, whose leading `..` leave its directory
            fs.create_symlink_file("/a/target", "../a/missing/../x.txt")
                .unwrap();
            assert_eq!(fs.read_to_string("/a/target").unwrap(), "x");
        }

        #[test]
        fn test_fs_should_resolve_relative_paths_against_current_dir() {
            let fs = TestFs::new();
            fs.create_dir("/tmp").unwrap();
            fs.env.replace_current_dir(root("/tmp"));

            fs.write("a.txt", "a").unwrap();

            assert_eq!(fs.current_dir(), root("/tmp"));
            assert_eq!(fs.read_to_string("/tmp/a.txt").unwrap(), "a");
            assert_eq!(fs.open_read("a.txt").unwrap().path(), root("/tmp/a.txt"));
        }

        #[test]
        fn test_fs_invalid_paths_should_fail() {
            let fs = TestFs::new();

            assert_eq!(kind(fs.metadata("")), ErrorKind::NotFound);
            assert_eq!(kind(fs.metadata("/a\0b")), ErrorKind::InvalidInput);
        }

        #[test]
        fn test_fs_symlink_loop_should_fail_after_max_symlinks() {
            // GIVEN: a chain of one symlink more than may be followed
            let fs = TestFs::new();
            fs.write("/a.txt", "").unwrap();
            let mut target = "a.txt".to_string();

            for i in 0..=MAX_SYMLINKS {
                let link = format!("link{i}");
                fs.create_symlink_file(format!("/{link}"), &target).unwrap();
                target = link;
            }

            // THEN: only the shorter chain resolves
            assert!(fs.metadata(format!("/link{}", MAX_SYMLINKS - 1)).is_ok());
            let error = fs.metadata(format!("/link{MAX_SYMLINKS}")).err().unwrap();
            assert_eq!(
                error.description().replace('\\', "/"),
                format!(
                    "Too many levels of symbolic links: {}",
                    shown(format!("/link{MAX_SYMLINKS}"))
                )
            );
            assert_eq!(
                error.source_as::<std::io::Error>().unwrap().kind(),
                symlink_loop_error().kind()
            );
        }

        #[test]
        fn test_fs_agdb_like_storage_should_read_positionally_and_survive_crash() {
            // GIVEN: a database file written and synced through one handle
            let fs = TestFs::new();
            let file = fs.open_write("/db").unwrap();
            file.write_all_at(b"0123456789", 0).unwrap();
            file.sync_data().unwrap();

            // WHEN: two threads read it positionally through the same handle
            std::thread::scope(|scope| {
                for offset in [0, 5] {
                    let file = &file;
                    scope.spawn(move || {
                        let mut buf = [0; 5];
                        file.read_exact_at(&mut buf, offset).unwrap();
                        assert_eq!(buf.as_slice(), &b"0123456789"[offset as usize..][..5]);
                    });
                }
            });

            // AND: a WAL is written, synced, written again and the system crashes
            let mut wal = fs.open_write("/db.wal").unwrap();
            wal.write_all(b"record 1;").unwrap();
            wal.sync_data().unwrap();
            wal.write_all(b"record 2;").unwrap();
            file.write_all_at(b"x", 0).unwrap();
            fs.crash();

            // THEN: only the synced data survives and the files reopen
            assert_eq!(fs.read_to_string("/db.wal").unwrap(), "record 1;");
            assert_eq!(fs.read_to_string("/db").unwrap(), "0123456789");
            drop(file);
            drop(wal);
            let _reopened = fs.open_write("/db").unwrap();
        }

        mod new {
            use super::*;

            #[test]
            fn new_should_hold_only_root_as_current_dir() {
                let fs = TestFs::new();

                assert_eq!(fs.current_dir(), Path::new(ROOT));
                assert!(fs.metadata(ROOT).unwrap().file_type.is_dir());
                assert_eq!(fs.read_dir(ROOT).unwrap().count(), 0);
            }
        }

        mod set_io_error {
            use super::*;

            #[test]
            fn set_io_error_should_fail_matching_paths_only() {
                let fs = TestFs::new();
                fs.create_dir("/data").unwrap();
                fs.set_io_error(FsOp::Open, "/data", ErrorKind::PermissionDenied);

                assert_eq!(
                    description(fs.write("/data/a.txt", "a")),
                    format!("simulated Open failure: {}", shown("/data/a.txt"))
                );
                assert_eq!(
                    kind(fs.write("/data/a.txt", "a")),
                    ErrorKind::PermissionDenied
                );
                assert!(fs.write("/database", "a").is_ok());
                assert!(fs.create_dir("/data/sub").is_ok());
            }

            #[test]
            fn set_io_error_empty_path_should_match_all() {
                let fs = TestFs::new();
                fs.set_io_error(FsOp::CreateDir, "", ErrorKind::Other);

                assert_eq!(kind(fs.create_dir("/a")), ErrorKind::Other);
                assert_eq!(kind(fs.create_dir_all("/b/c")), ErrorKind::Other);
            }

            #[test]
            fn set_io_error_relative_path_should_be_resolved_against_current_dir() {
                let fs = TestFs::new();
                fs.create_dir("/tmp").unwrap();
                fs.env.replace_current_dir(root("/tmp"));
                fs.set_io_error(FsOp::Open, "a.txt", ErrorKind::Other);

                assert!(fs.write("/tmp/a.txt", "a").is_err());
                assert!(fs.write("/a.txt", "a").is_ok());
            }

            #[test]
            fn set_io_error_should_match_resolved_path() {
                let fs = TestFs::new();
                fs.create_dir("/data").unwrap();
                fs.create_symlink_dir("/link", "/data").unwrap();
                fs.set_io_error(FsOp::Open, "/data", ErrorKind::Other);

                assert!(fs.write("/link/a.txt", "a").is_err());
            }

            #[test]
            fn set_io_error_path_with_parent_dir_should_match_normalized_path() {
                let fs = TestFs::new();
                fs.set_io_error(FsOp::Open, "/a/../b.txt", ErrorKind::Other);

                assert!(fs.write("/b.txt", "b").is_err());
                assert!(fs.write("/a.txt", "a").is_ok());
            }

            #[test]
            fn set_io_error_most_recent_should_win() {
                let fs = TestFs::new();
                fs.set_io_error(FsOp::Open, "", ErrorKind::Other);
                fs.set_io_errors(FsOp::Open, "", vec![]);
                fs.set_io_errors(FsOp::Open, "/a.txt", vec![None]);

                assert!(fs.write("/a.txt", "a").is_ok());
                assert!(fs.write("/b.txt", "b").is_err());
            }

            #[test]
            fn set_io_error_should_apply_to_handles_opened_before() {
                let fs = TestFs::new();
                let mut file = fs.open_write("/a.txt").unwrap();
                fs.set_io_error(FsOp::Write, "/a.txt", ErrorKind::StorageFull);

                assert_eq!(file.write(b"a").unwrap_err().kind(), ErrorKind::StorageFull);
            }
        }

        mod set_io_errors {
            use super::*;

            #[test]
            fn set_io_errors_should_return_one_result_per_call_and_repeat_last() {
                let fs = TestFs::new();
                fs.set_io_errors(FsOp::Metadata, "", [None, Some(ErrorKind::Other), None]);

                assert!(fs.exists("/").is_ok());
                assert!(fs.exists("/").is_err());
                assert!(fs.exists("/").is_ok());
                assert!(fs.exists("/").is_ok());
            }

            #[test]
            fn set_io_errors_empty_should_set_nothing() {
                let fs = TestFs::new();
                fs.set_io_error(FsOp::Metadata, "", ErrorKind::Other);
                fs.set_io_errors(FsOp::Metadata, "", Vec::new());

                assert!(fs.exists("/").is_err());
            }
        }

        mod set_short_io {
            use super::*;

            #[test]
            fn set_short_io_should_limit_reads_and_writes() {
                let fs = TestFs::new();
                fs.set_short_io(FsOp::Write, "", 2);
                fs.set_short_io(FsOp::Read, "", 3);
                let mut file = fs.open_write("/a.txt").unwrap();

                assert_eq!(file.write(b"hello").unwrap(), 2);
                file.write_all(b"llo").unwrap();
                file.seek(SeekFrom::Start(0)).unwrap();
                let mut buf = [0; 5];
                assert_eq!(file.read(&mut buf).unwrap(), 3);
                assert_eq!(&buf[..3], b"hel");
                let mut rest = Vec::new();
                assert_eq!(file.read_to_end(&mut rest).unwrap(), 2);
                assert_eq!(rest, b"lo");
            }

            #[test]
            fn set_short_io_should_still_read_to_end_of_file() {
                let fs = TestFs::new();
                fs.write("/a.txt", "hello").unwrap();
                fs.set_short_io(FsOp::Read, "", 2);

                assert_eq!(fs.read("/a.txt").unwrap(), b"hello");
            }

            #[test]
            fn set_short_io_of_zero_bytes_should_read_nothing_to_end() {
                let fs = TestFs::new();
                fs.write("/a.txt", "hello").unwrap();
                fs.set_short_io(FsOp::Read, "", 0);

                assert_eq!(fs.read("/a.txt").unwrap(), b"");
            }

            #[test]
            fn set_short_io_other_operation_should_be_ignored() {
                let fs = TestFs::new();
                fs.set_short_io(FsOp::Open, "", 0);

                fs.write("/a.txt", "hello").unwrap();

                assert_eq!(fs.read_to_string("/a.txt").unwrap(), "hello");
            }
        }

        mod set_capacity {
            use super::*;

            #[test]
            fn set_capacity_should_fail_growing_beyond_it() {
                // GIVEN: 5 of 8 bytes used
                let fs = TestFs::new();
                fs.set_capacity(Some(8));
                fs.write("/a.txt", "hello").unwrap();
                let file = fs.open_write("/b.txt").unwrap();

                // THEN: growing beyond 8 bytes fails, up to it succeeds
                assert_eq!(kind(file.write_at(b"abcd", 0)), ErrorKind::StorageFull);
                assert_eq!(kind(file.set_len(4)), ErrorKind::StorageFull);
                assert_eq!(file.write_at(b"abc", 0).unwrap(), 3);
                drop(file);
                assert_eq!(kind(fs.copy("/a.txt", "/c.txt")), ErrorKind::StorageFull);
                assert_eq!(kind(fs.copy("/a.txt", "/b.txt")), ErrorKind::StorageFull);
            }

            #[test]
            fn set_capacity_should_allow_shrinking_beyond_it() {
                let fs = TestFs::new();
                fs.write("/a.txt", "hello").unwrap();
                fs.set_capacity(Some(1));
                let file = fs.open_write("/a.txt").unwrap();

                file.set_len(2).unwrap();

                assert_eq!(file.metadata().unwrap().len, 2);
            }

            #[test]
            fn set_capacity_none_should_remove_limit() {
                let fs = TestFs::new();
                fs.set_capacity(Some(1));
                fs.set_capacity(None);

                fs.write("/a.txt", "hello").unwrap();
            }
        }

        mod clear_faults {
            use super::*;

            #[test]
            fn clear_faults_should_remove_all_faults() {
                let fs = TestFs::new();
                fs.set_io_error(FsOp::Open, "", ErrorKind::Other);
                fs.set_capacity(Some(0));

                fs.clear_faults();

                fs.write("/a.txt", "a").unwrap();
            }
        }

        mod crash {
            use super::*;

            #[test]
            fn crash_should_revert_files_to_last_sync() {
                // GIVEN: a synced file written again and a file never synced
                let fs = TestFs::new();
                let mut file = fs.open_write("/a.txt").unwrap();
                file.write_all(b"synced").unwrap();
                file.sync_all().unwrap();
                file.write_all(b" lost").unwrap();
                fs.write("/b.txt", "never synced").unwrap();

                // WHEN
                fs.crash();

                // THEN
                assert_eq!(fs.read_to_string("/a.txt").unwrap(), "synced");
                assert_eq!(fs.read_to_string("/b.txt").unwrap(), "");
            }

            #[test]
            fn crash_should_close_handles_and_release_borrows() {
                // GIVEN: a file open for writing
                let fs = TestFs::new();
                let mut old = fs.open_write("/a.txt").unwrap();

                // WHEN: the system crashes and the file is opened again
                fs.crash();
                let new = fs.open_write("/a.txt").unwrap();

                // THEN: the old handle fails and dropping it keeps the new borrow
                assert_eq!(
                    old.write(b"a").unwrap_err().to_string(),
                    "TestFs crashed, the file handle is closed"
                );
                assert_eq!(
                    description(old.metadata()),
                    format!(
                        "TestFs crashed, the file handle is closed: {}",
                        shown("/a.txt")
                    )
                );
                for error in [
                    old.read(&mut [0]).unwrap_err().to_string(),
                    old.read_to_end(&mut Vec::new()).unwrap_err().to_string(),
                    description(old.set_len(1)),
                    description(old.sync_data()),
                ] {
                    assert!(error.starts_with("TestFs crashed"), "{error}");
                }
                drop(old);
                assert_eq!(state(&fs, "/a.txt"), Some(BorrowState::Write));
                drop(new);
                assert_eq!(state(&fs, "/a.txt"), Some(BorrowState::Read(0)));
            }
        }

        mod sync {
            use super::*;

            #[test]
            fn sync_should_make_content_survive_crash() {
                let fs = TestFs::new();
                fs.create_dir("/dir").unwrap();
                fs.write("/a.txt", "a").unwrap();

                fs.sync();
                fs.crash();

                assert_eq!(fs.read_to_string("/a.txt").unwrap(), "a");
            }
        }

        mod set_created {
            use super::*;

            #[test]
            fn set_created_unresolvable_path_should_fail() {
                let fs = looped();

                assert_eq!(
                    kind(fs.set_created("", SystemTime::UNIX_EPOCH)),
                    ErrorKind::NotFound
                );
            }

            #[test]
            fn set_created_should_set_creation_time() {
                let fs = TestFs::new();
                fs.write("/a.txt", "").unwrap();
                fs.create_dir("/dir").unwrap();

                fs.set_created("/a.txt", SystemTime::UNIX_EPOCH).unwrap();
                fs.set_created("/dir", SystemTime::UNIX_EPOCH).unwrap();

                assert_eq!(
                    fs.metadata("/a.txt").unwrap().created,
                    SystemTime::UNIX_EPOCH
                );
                assert_eq!(fs.metadata("/dir").unwrap().created, SystemTime::UNIX_EPOCH);
            }

            #[test]
            fn set_created_missing_path_should_fail() {
                let fs = TestFs::new();
                fs.create_symlink_file("/link", "/missing").unwrap();

                assert_eq!(
                    description(fs.set_created("/link", SystemTime::UNIX_EPOCH)),
                    format!(
                        "Creation time could not be set, path not found: {}",
                        shown("/link")
                    )
                );
            }
        }

        mod set_len {
            use super::*;

            #[test]
            fn set_len_should_extend_file_sparsely() {
                let fs = TestFs::new();
                fs.write("/a.txt", "abc").unwrap();

                fs.set_len("/a.txt", 10 << 30).unwrap();
                fs.set_len("/a.txt", 5).unwrap();

                assert_eq!(fs.read("/a.txt").unwrap(), b"abc\0\0");
            }

            #[test]
            fn set_len_invalid_should_fail() {
                let fs = TestFs::new();
                fs.write("/a.txt", "").unwrap();
                fs.create_dir("/dir").unwrap();

                assert_eq!(
                    kind(fs.set_len("/a.txt", u64::MAX)),
                    ErrorKind::InvalidInput
                );
                assert_eq!(kind(fs.set_len("/dir", 1)), ErrorKind::IsADirectory);
                assert_eq!(kind(fs.set_len("/missing", 1)), ErrorKind::NotFound);
            }
        }

        #[cfg(unix)]
        mod set_owner {
            use super::*;

            #[test]
            fn set_owner_unresolvable_path_should_fail() {
                let fs = looped();

                assert_eq!(kind(fs.set_owner("", 0, 0)), ErrorKind::NotFound);
            }

            #[test]
            fn set_owner_should_set_owner_of_file_and_dir() {
                let fs = TestFs::new();
                fs.write("/a.txt", "").unwrap();
                fs.create_dir("/dir").unwrap();

                fs.set_owner("/a.txt", 0, 1).unwrap();
                fs.set_owner("/dir", 2, 3).unwrap();

                let file = fs.metadata("/a.txt").unwrap();
                let dir = fs.metadata("/dir").unwrap();
                assert_eq!((file.uid, file.gid, dir.uid, dir.gid), (0, 1, 2, 3));
            }

            #[test]
            fn set_owner_missing_path_should_fail() {
                let fs = TestFs::new();

                assert_eq!(
                    description(fs.set_owner("/missing", 0, 0)),
                    format!(
                        "Attributes could not be set, path not found: {}",
                        shown("/missing")
                    )
                );
            }
        }

        #[cfg(windows)]
        mod set_hidden {
            use super::*;

            #[test]
            fn set_hidden_should_set_and_clear_attribute_of_file_and_dir() {
                let fs = TestFs::new();
                fs.write("/a.txt", "").unwrap();
                fs.create_dir("/dir").unwrap();

                for hidden in [true, false] {
                    fs.set_hidden("/a.txt", hidden).unwrap();
                    fs.set_hidden("/dir", hidden).unwrap();

                    assert!(
                        fs.metadata("/a.txt").unwrap().hidden == hidden
                            && fs.metadata("/dir").unwrap().hidden == hidden,
                        "hidden: {hidden}"
                    );
                }
            }

            #[test]
            fn set_hidden_unresolvable_path_should_fail() {
                let fs = looped();

                assert!(fs.set_hidden("/loop", true).is_err());
            }

            #[test]
            fn set_hidden_missing_path_should_fail() {
                let fs = TestFs::new();

                assert_eq!(kind(fs.set_hidden("/missing", true)), ErrorKind::NotFound);
            }
        }

        mod set_env {
            use super::*;

            #[test]
            fn set_env_should_resolve_against_its_current_dir() {
                let mut fs = TestFs::new();
                fs.create_dir("/home").unwrap();

                fs.set_env(EnvImpl::new(Vec::new(), root("/home")));
                fs.write("a.txt", "a").unwrap();

                assert!(fs.exists("/home/a.txt").unwrap());
            }
        }

        mod fs {
            use super::*;

            mod open_read {
                use super::*;

                #[test]
                fn open_read_should_open_relative_and_absolute_path() {
                    crate::fs::tests::fs::open_read::open_read_should_open_relative_and_absolute_path(
                        &TestFs::new(),
                        Path::new(ROOT),
                    );
                }

                #[test]
                fn open_read_missing_file_should_fail_with_path() {
                    crate::fs::tests::fs::open_read::open_read_missing_file_should_fail_with_path(
                        &TestFs::new(),
                        Path::new(ROOT),
                    );
                }

                #[test]
                fn open_read_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.open_read("")), ErrorKind::NotFound);
                }

                #[test]
                fn open_read_dir_should_fail() {
                    let fs = TestFs::new();
                    fs.create_dir("/dir").unwrap();

                    assert_eq!(
                        description(fs.open_read("/dir")),
                        format!(
                            "File could not be opened, path is a directory: {}",
                            shown("/dir")
                        )
                    );
                }

                #[test]
                fn open_read_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "").unwrap();
                    fs.set_io_error(FsOp::Open, "/a.txt", ErrorKind::Other);

                    assert_eq!(kind(fs.open_read("/a.txt")), ErrorKind::Other);
                }
            }

            mod open_write {
                use super::*;

                #[test]
                fn open_write_should_create_file_without_truncating() {
                    crate::fs::tests::fs::open_write::open_write_should_create_file_without_truncating(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn open_write_should_open_for_reading() {
                    crate::fs::tests::fs::open_write::open_write_should_open_for_reading(
                        &TestFs::new(),
                    );
                }
            }

            mod open_write_options {
                use super::*;

                #[test]
                fn open_write_options_should_append_or_truncate() {
                    crate::fs::tests::fs::open_write_options::open_write_options_should_append_or_truncate(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn open_write_options_append_should_write_at_end_regardless_of_position() {
                    crate::fs::tests::fs::open_write_options::open_write_options_append_should_write_at_end_regardless_of_position(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn open_write_options_write_only_should_not_be_readable() {
                    crate::fs::tests::fs::open_write_options::open_write_options_write_only_should_not_be_readable(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn open_write_options_without_create_missing_file_should_fail() {
                    crate::fs::tests::fs::open_write_options::open_write_options_without_create_missing_file_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn open_write_options_create_new_existing_should_fail() {
                    crate::fs::tests::fs::open_write_options::open_write_options_create_new_existing_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn open_write_options_append_and_truncate_should_require_create_new() {
                    crate::fs::tests::fs::open_write_options::open_write_options_append_and_truncate_should_require_create_new(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn open_write_options_create_should_follow_dangling_symlink() {
                    crate::fs::tests::fs::open_write_options::open_write_options_create_should_follow_dangling_symlink(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn open_write_options_create_with_separator_should_fail() {
                    crate::fs::tests::fs::open_write_options::open_write_options_create_with_separator_should_fail(
                        &TestFs::new(),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn open_write_options_mode_should_set_mode_of_new_file() {
                    crate::fs::tests::fs::open_write_options::open_write_options_mode_should_set_mode_of_new_file(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn open_write_options_invalid_target_should_fail() {
                    crate::fs::tests::fs::open_write_options::open_write_options_invalid_target_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn open_write_options_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.open_write("")), ErrorKind::NotFound);
                    assert!(fs.open_write("/loop/x/").is_err());
                    assert!(fs.open_write("/loop/x").is_err());
                    assert!(fs.open_write("/loop").is_err());
                }

                #[test]
                fn open_write_options_readonly_file_should_fail() {
                    let fs = TestFs::new();
                    fs.write("/ro.txt", "").unwrap();
                    fs.set_readonly("/ro.txt", true).unwrap();

                    assert_eq!(
                        description(fs.open_write("/ro.txt")),
                        format!(
                            "File could not be opened, it is read-only: {}",
                            shown("/ro.txt")
                        )
                    );
                }

                #[test]
                fn open_write_options_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::Open, "/a.txt", ErrorKind::Other);

                    assert_eq!(kind(fs.open_write("/a.txt")), ErrorKind::Other);
                    assert!(!fs.exists("/a.txt").unwrap());
                }
            }

            mod read {
                use super::*;

                #[test]
                fn read_should_return_content() {
                    crate::fs::tests::fs::read::read_should_return_content(&TestFs::new());
                }

                #[test]
                fn read_missing_file_should_fail_with_path() {
                    crate::fs::tests::fs::read::read_missing_file_should_fail_with_path(
                        &TestFs::new(),
                        Path::new(ROOT),
                    );
                }

                #[test]
                fn read_directory_should_fail() {
                    crate::fs::tests::fs::read::read_directory_should_fail(&TestFs::new());
                }

                #[test]
                fn read_failing_read_should_fail_with_path() {
                    let fs = TestFs::new();
                    fs.write("/a.bin", [1]).unwrap();
                    fs.set_io_error(FsOp::Read, "", ErrorKind::Other);

                    assert_eq!(
                        description(fs.read("/a.bin")),
                        format!("simulated Read failure: {}", shown("/a.bin"))
                    );
                }
            }

            mod read_to_string {
                use super::*;

                #[test]
                fn read_to_string_should_return_content() {
                    crate::fs::tests::fs::read_to_string::read_to_string_should_return_content(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn read_to_string_invalid_utf8_should_fail_as_invalid_data() {
                    crate::fs::tests::fs::read_to_string::read_to_string_invalid_utf8_should_fail_as_invalid_data(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn read_to_string_failing_read_should_fail() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "abc").unwrap();
                    fs.set_io_error(FsOp::Read, "", ErrorKind::Other);

                    assert_eq!(kind(fs.read_to_string("/a.txt")), ErrorKind::Other);
                }
            }

            mod write {
                use super::*;

                #[test]
                fn write_should_replace_content() {
                    crate::fs::tests::fs::write::write_should_replace_content(&TestFs::new());
                }

                #[cfg(unix)]
                #[test]
                fn write_should_write_file_that_cannot_be_read() {
                    crate::fs::tests::fs::write::write_should_write_file_that_cannot_be_read(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn write_missing_parent_should_fail() {
                    crate::fs::tests::fs::write::write_missing_parent_should_fail(&TestFs::new());
                }

                #[test]
                fn write_directory_should_fail() {
                    crate::fs::tests::fs::write::write_directory_should_fail(&TestFs::new());
                }

                #[test]
                fn write_should_update_modified_time() {
                    crate::fs::tests::fs::write::write_should_update_modified_time(&TestFs::new());
                }

                #[test]
                fn write_new_file_should_update_modified_time_of_parent() {
                    crate::fs::tests::fs::write::write_new_file_should_update_modified_time_of_parent(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn write_path_with_nul_byte_should_fail() {
                    crate::fs::tests::fs::write::write_path_with_nul_byte_should_fail(
                        &TestFs::new(),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn write_file_with_trailing_current_dir_should_fail() {
                    crate::fs::tests::fs::write::write_file_with_trailing_current_dir_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn write_failing_write_should_fail_with_path() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::Write, "", ErrorKind::StorageFull);

                    assert_eq!(
                        description(fs.write("/a.txt", "a")),
                        format!("simulated Write failure: {}", shown("/a.txt"))
                    );
                }
            }

            mod read_dir {
                use super::*;

                #[test]
                fn read_dir_should_list_entries_under_given_path() {
                    crate::fs::tests::fs::read_dir::read_dir_should_list_entries_under_given_path(
                        &TestFs::new(),
                        Path::new(ROOT),
                    );
                }

                #[test]
                fn read_dir_invalid_path_should_fail() {
                    crate::fs::tests::fs::read_dir::read_dir_invalid_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn read_dir_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.read_dir("")), ErrorKind::NotFound);
                }

                #[test]
                fn read_dir_should_list_entries_in_hash_order() {
                    let fs = TestFs::new();
                    fs.create_dir_all("/dir/sub").unwrap();
                    fs.write("/dir/a.txt", "").unwrap();
                    fs.create_symlink_file("/dir/link", "/missing").unwrap();

                    let entries: Vec<DirEntry> =
                        fs.read_dir("/dir").unwrap().collect::<Result<_>>().unwrap();

                    let mut expected = vec![
                        DirEntry::new(root("/dir/sub"), FileType::Dir),
                        DirEntry::new(root("/dir/a.txt"), FileType::File),
                        DirEntry::new(root("/dir/link"), FileType::Symlink),
                    ];
                    expected.sort_by_key(|entry| name_hash(entry.file_name()));
                    assert_eq!(entries, expected);
                }

                #[test]
                fn read_dir_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::ReadDir, "", ErrorKind::Other);

                    assert_eq!(kind(fs.read_dir("/")), ErrorKind::Other);
                }
            }

            mod create_dir {
                use super::*;

                #[test]
                fn create_dir_should_create_only_new_dir() {
                    crate::fs::tests::fs::create_dir::create_dir_should_create_only_new_dir(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_dir_invalid_path_should_fail() {
                    crate::fs::tests::fs::create_dir::create_dir_invalid_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn create_dir_with_trailing_current_dir_should_fail() {
                    crate::fs::tests::fs::create_dir::create_dir_with_trailing_current_dir_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_dir_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.create_dir("")), ErrorKind::NotFound);
                    assert!(fs.create_dir("/loop/x").is_err());
                    assert!(fs.create_dir("/loop/.").is_err());
                }

                #[test]
                fn create_dir_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::CreateDir, "/dir", ErrorKind::Other);

                    assert_eq!(kind(fs.create_dir("/dir")), ErrorKind::Other);
                }
            }

            mod create_dir_all {
                use super::*;

                #[test]
                fn create_dir_all_should_create_missing_parents() {
                    crate::fs::tests::fs::create_dir_all::create_dir_all_should_create_missing_parents(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_dir_all_existing_non_dir_should_fail() {
                    crate::fs::tests::fs::create_dir_all::create_dir_all_existing_non_dir_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_dir_all_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.create_dir_all("")), ErrorKind::NotFound);
                    assert!(fs.create_dir_all("/loop/x").is_err());
                }

                #[test]
                fn create_dir_all_through_dangling_symlink_should_fail_naming_it() {
                    let fs = TestFs::new();
                    fs.create_symlink_file("/link", "/missing").unwrap();

                    assert_eq!(
                        description(fs.create_dir_all("/link/b")),
                        format!(
                            "Directory could not be created, a dangling symlink exists at: {}",
                            shown("/link")
                        )
                    );
                }

                #[test]
                fn create_dir_all_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::CreateDir, "", ErrorKind::Other);

                    assert_eq!(kind(fs.create_dir_all("/a")), ErrorKind::Other);
                }
            }

            mod exists {
                use super::*;

                #[test]
                fn exists_should_follow_symlinks() {
                    crate::fs::tests::fs::exists::exists_should_follow_symlinks(&TestFs::new());
                }

                #[cfg(unix)]
                #[test]
                fn exists_file_as_parent_should_fail() {
                    crate::fs::tests::fs::exists::exists_file_as_parent_should_fail(&TestFs::new());
                }

                #[cfg(windows)]
                #[test]
                fn exists_file_as_parent_should_not_exist() {
                    crate::fs::tests::fs::exists::exists_file_as_parent_should_not_exist(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn exists_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.exists("")), ErrorKind::NotFound);
                }

                #[test]
                fn exists_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::Metadata, "", ErrorKind::Other);

                    assert_eq!(kind(fs.exists("/")), ErrorKind::Other);
                }
            }

            mod metadata {
                use super::*;

                #[test]
                fn metadata_should_follow_symlinks() {
                    crate::fs::tests::fs::metadata::metadata_should_follow_symlinks(&TestFs::new());
                }

                #[test]
                fn metadata_missing_path_should_fail() {
                    crate::fs::tests::fs::metadata::metadata_missing_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn metadata_file_with_trailing_current_dir_should_fail() {
                    crate::fs::tests::fs::metadata::metadata_file_with_trailing_current_dir_should_fail(
                        &TestFs::new(),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn metadata_should_report_mode_of_new_entries() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "").unwrap();

                    assert_eq!(fs.metadata("/a.txt").unwrap().mode, 0o644);
                    assert_eq!(fs.metadata("/").unwrap().mode, 0o755);
                }

                #[test]
                fn metadata_missing_should_fail() {
                    let fs = TestFs::new();

                    assert_eq!(
                        description(fs.metadata("/missing")),
                        format!(
                            "Metadata could not be read, path not found: {}",
                            shown("/missing")
                        )
                    );
                }

                #[test]
                fn metadata_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::Metadata, "", ErrorKind::Other);

                    assert_eq!(kind(fs.metadata("/")), ErrorKind::Other);
                }
            }

            mod symlink_metadata {
                use super::*;

                #[test]
                fn symlink_metadata_should_not_follow_symlink() {
                    crate::fs::tests::fs::symlink_metadata::symlink_metadata_should_not_follow_symlink(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn symlink_metadata_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.symlink_metadata("")), ErrorKind::NotFound);
                }

                #[test]
                fn symlink_metadata_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::Metadata, "", ErrorKind::Other);

                    assert_eq!(kind(fs.symlink_metadata("/")), ErrorKind::Other);
                }
            }

            mod create_symlink_file {
                use super::*;

                #[test]
                fn create_symlink_file_should_resolve_relative_target_against_link_parent() {
                    crate::fs::tests::fs::create_symlink_file::create_symlink_file_should_resolve_relative_target_against_link_parent(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_symlink_file_with_parent_dir_in_target_should_resolve_per_os() {
                    crate::fs::tests::fs::create_symlink_file::create_symlink_file_with_parent_dir_in_target_should_resolve_per_os(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_symlink_file_current_dir_target_should_resolve() {
                    crate::fs::tests::fs::create_symlink_file::create_symlink_file_current_dir_target_should_resolve(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_symlink_file_should_allow_dangling_target() {
                    crate::fs::tests::fs::create_symlink_file::create_symlink_file_should_allow_dangling_target(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_symlink_file_existing_path_should_fail() {
                    crate::fs::tests::fs::create_symlink_file::create_symlink_file_existing_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_symlink_file_invalid_path_should_fail() {
                    crate::fs::tests::fs::create_symlink_file::create_symlink_file_invalid_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_symlink_file_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.create_symlink_file("", "a")), ErrorKind::NotFound);
                    assert!(fs.create_symlink_file("/loop/x", "a").is_err());
                }

                #[test]
                fn create_symlink_file_dangling_target_should_be_created_as_file() {
                    let fs = TestFs::new();

                    fs.create_symlink_file("/link", "a.txt").unwrap();
                    fs.write("/a.txt", "a").unwrap();

                    assert!(!symlink(&fs, "/link").unwrap().dir);
                    assert!(symlink(&fs, "/a.txt").is_none());
                    assert_eq!(state(&fs, "/link"), None);
                }

                #[test]
                fn create_symlink_file_to_dir_should_be_file_symlink() {
                    let fs = TestFs::new();
                    fs.create_dir("/dir").unwrap();

                    fs.create_symlink_file("/link", "dir").unwrap();

                    assert!(!symlink(&fs, "/link").unwrap().dir);
                }

                #[test]
                fn create_symlink_file_invalid_path_should_fail_naming_it() {
                    let fs = TestFs::new();

                    assert_eq!(
                        description(fs.create_symlink_file("/link/", "b")),
                        format!(
                            "Symlink could not be created, path ends with a separator: {}",
                            shown("/link/")
                        )
                    );
                    assert_eq!(
                        description(fs.create_symlink_file("/link", "")),
                        format!(
                            "Symlink could not be created, target is empty: {}",
                            shown("/link")
                        )
                    );
                }

                #[test]
                fn create_symlink_file_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::CreateLink, "", ErrorKind::Other);

                    assert_eq!(kind(fs.create_symlink_file("/link", "a")), ErrorKind::Other);
                }
            }

            mod create_symlink_dir {
                use super::*;

                #[test]
                fn create_symlink_dir_should_link_dir_created_later() {
                    crate::fs::tests::fs::create_symlink_dir::create_symlink_dir_should_link_dir_created_later(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_symlink_dir_should_be_dir_symlink_to_missing_target() {
                    let fs = TestFs::new();

                    fs.create_symlink_dir("/link", "/dir").unwrap();

                    assert!(symlink(&fs, "/link").unwrap().dir);
                }
            }

            mod read_link {
                use super::*;

                #[test]
                fn read_link_should_return_target_as_stored() {
                    crate::fs::tests::fs::read_link::read_link_should_return_target_as_stored(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn read_link_not_symlink_should_fail() {
                    crate::fs::tests::fs::read_link::read_link_not_symlink_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn read_link_missing_path_should_fail() {
                    crate::fs::tests::fs::read_link::read_link_missing_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn read_link_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.read_link("")), ErrorKind::NotFound);
                }

                #[test]
                fn read_link_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::ReadLink, "/fault", ErrorKind::Other);

                    assert_eq!(kind(fs.read_link("/fault")), ErrorKind::Other);
                }
            }

            mod canonicalize {
                use super::*;

                #[test]
                fn canonicalize_should_resolve_symlinks_and_parent_dirs() {
                    crate::fs::tests::fs::canonicalize::canonicalize_should_resolve_symlinks_and_parent_dirs(
                        &TestFs::new(),
                        Path::new(ROOT),
                    );
                }

                #[test]
                fn canonicalize_missing_path_should_fail() {
                    crate::fs::tests::fs::canonicalize::canonicalize_missing_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn canonicalize_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.canonicalize("")), ErrorKind::NotFound);
                }

                #[test]
                fn canonicalize_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::Canonicalize, "/fault", ErrorKind::Other);

                    assert_eq!(kind(fs.canonicalize("/fault")), ErrorKind::Other);
                }
            }

            mod create_hard_link {
                use super::*;

                #[test]
                fn create_hard_link_should_share_content() {
                    crate::fs::tests::fs::create_hard_link::create_hard_link_should_share_content(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_hard_link_to_symlink_should_link_symlink_itself() {
                    crate::fs::tests::fs::create_hard_link::create_hard_link_to_symlink_should_link_symlink_itself(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_hard_link_invalid_path_should_fail() {
                    crate::fs::tests::fs::create_hard_link::create_hard_link_invalid_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn create_hard_link_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.create_hard_link("/a", "")), ErrorKind::NotFound);
                    assert_eq!(kind(fs.create_hard_link("", "/a")), ErrorKind::NotFound);
                    assert!(fs.create_hard_link("/loop/x", "/a").is_err());
                    assert!(fs.create_hard_link("/a", "/loop/x").is_err());
                }

                #[test]
                fn create_hard_link_path_with_separator_should_fail_naming_it() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "").unwrap();

                    assert_eq!(
                        description(fs.create_hard_link("/b/", "/a.txt")),
                        format!(
                            "Hard link could not be created, path ends with a separator: {}",
                            shown("/b/")
                        )
                    );
                }

                #[test]
                fn create_hard_link_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "").unwrap();
                    fs.set_io_error(FsOp::CreateLink, "", ErrorKind::Other);

                    assert_eq!(
                        kind(fs.create_hard_link("/b.txt", "/a.txt")),
                        ErrorKind::Other
                    );
                }
            }

            mod copy {
                use super::*;

                #[test]
                fn copy_should_overwrite_target() {
                    crate::fs::tests::fs::copy::copy_should_overwrite_target(&TestFs::new());
                }

                #[test]
                fn copy_should_copy_content_and_permissions_through_symlinks() {
                    crate::fs::tests::fs::copy::copy_should_copy_content_and_permissions_through_symlinks(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn copy_should_follow_dangling_symlink_target() {
                    crate::fs::tests::fs::copy::copy_should_follow_dangling_symlink_target(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn copy_onto_itself_should_fail_keeping_content() {
                    crate::fs::tests::fs::copy::copy_onto_itself_should_fail_keeping_content(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn copy_dir_should_fail() {
                    crate::fs::tests::fs::copy::copy_dir_should_fail(&TestFs::new());
                }

                #[test]
                fn copy_missing_source_should_fail() {
                    crate::fs::tests::fs::copy::copy_missing_source_should_fail(&TestFs::new());
                }

                #[test]
                fn copy_invalid_target_should_fail() {
                    crate::fs::tests::fs::copy::copy_invalid_target_should_fail(&TestFs::new());
                }

                #[test]
                fn copy_unresolvable_path_should_fail() {
                    let fs = looped();

                    fs.write("/a.txt", "").unwrap();
                    assert_eq!(kind(fs.copy("", "/b")), ErrorKind::NotFound);
                    assert_eq!(kind(fs.copy("/a.txt", "")), ErrorKind::NotFound);
                    assert!(fs.copy("/loop/x", "/b").is_err());
                    assert!(fs.copy("/a.txt", "/loop/x/").is_err());
                    assert!(fs.copy("/a.txt", "/loop").is_err());
                }

                #[cfg(unix)]
                #[test]
                fn copy_should_give_copy_new_owner() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "hello").unwrap();
                    fs.set_owner("/a.txt", 0, 0).unwrap();

                    fs.copy("/a.txt", "/b.txt").unwrap();

                    assert_eq!(fs.metadata("/b.txt").unwrap().uid, OWNER_ID);
                }

                #[cfg(windows)]
                #[test]
                fn copy_should_create_independent_copy_with_attributes() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "hello").unwrap();
                    fs.set_hidden("/a.txt", true).unwrap();
                    fs.create_symlink_file("/link", "a.txt").unwrap();

                    assert_eq!(fs.copy("/link", "/b.txt").unwrap(), 5);
                    fs.write("/b.txt", "world").unwrap();

                    assert_eq!(fs.read_to_string("/a.txt").unwrap(), "hello");
                    assert!(fs.metadata("/b.txt").unwrap().hidden);
                }

                #[cfg(unix)]
                #[test]
                fn copy_should_overwrite_target_keeping_owner() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "hello").unwrap();
                    fs.set_mode("/a.txt", 0o600).unwrap();
                    fs.write("/b.txt", "old content").unwrap();
                    fs.set_owner("/b.txt", 0, 0).unwrap();
                    fs.create_symlink_file("/link", "b.txt").unwrap();
                    let _read = fs.open_read("/a.txt").unwrap();

                    fs.copy("/a.txt", "/link").unwrap();

                    assert_eq!(fs.read_to_string("/b.txt").unwrap(), "hello");
                    let metadata = fs.metadata("/b.txt").unwrap();
                    assert_eq!((metadata.mode, metadata.uid), (0o600, 0));
                }

                #[cfg(windows)]
                #[test]
                fn copy_should_overwrite_target_with_all_attributes() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "hello").unwrap();
                    fs.set_hidden("/a.txt", true).unwrap();
                    fs.write("/b.txt", "old content").unwrap();

                    fs.copy("/a.txt", "/b.txt").unwrap();

                    assert_eq!(fs.read_to_string("/b.txt").unwrap(), "hello");
                    assert!(fs.metadata("/b.txt").unwrap().hidden);
                    fs.write("/a.txt", "new").unwrap();
                    fs.copy("/a.txt", "/b.txt").unwrap();
                    assert_eq!(fs.read_to_string("/b.txt").unwrap(), "new");
                }

                #[cfg(windows)]
                #[test]
                fn copy_not_hidden_over_hidden_target_should_fail() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "hello").unwrap();
                    fs.write("/b.txt", "old content").unwrap();
                    fs.set_hidden("/b.txt", true).unwrap();

                    let result = fs.copy("/a.txt", "/b.txt");

                    assert_eq!(kind(result), ErrorKind::PermissionDenied);
                    assert_eq!(fs.read_to_string("/b.txt").unwrap(), "old content");
                }

                #[test]
                fn copy_borrowed_source_should_fail() {
                    let fs = TestFs::new();
                    let _write = fs.open_write("/w.txt").unwrap();

                    assert_eq!(kind(fs.copy("/w.txt", "/b")), ErrorKind::ResourceBusy);
                }

                #[test]
                fn copy_unwritable_target_should_fail() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "a").unwrap();
                    fs.write("/ro.txt", "").unwrap();
                    fs.set_readonly("/ro.txt", true).unwrap();
                    let _read = fs.open_read("/a.txt").unwrap();

                    assert_eq!(
                        kind(fs.copy("/a.txt", "/ro.txt")),
                        ErrorKind::PermissionDenied
                    );
                    assert_eq!(
                        description(fs.copy("/a.txt", "/a.txt")),
                        format!(
                            "File could not be opened, it is already borrowed for reading 2 times: {}",
                            shown("/a.txt")
                        )
                    );
                }

                #[test]
                fn copy_borrowed_target_should_fail() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "a").unwrap();
                    let _write = fs.open_write("/b.txt").unwrap();

                    assert_eq!(kind(fs.copy("/a.txt", "/b.txt")), ErrorKind::ResourceBusy);
                }

                #[test]
                fn copy_fault_should_fail() {
                    for op in [FsOp::Open, FsOp::Read, FsOp::Write] {
                        let fs = TestFs::new();
                        fs.write("/a.txt", "a").unwrap();
                        fs.set_io_error(op, "", ErrorKind::Other);

                        assert_eq!(
                            kind(fs.copy("/a.txt", "/b.txt")),
                            ErrorKind::Other,
                            "{op:?}"
                        );
                    }

                    let fs = TestFs::new();
                    fs.write("/a.txt", "a").unwrap();
                    fs.set_io_error(FsOp::Open, "/b.txt", ErrorKind::Other);
                    assert_eq!(kind(fs.copy("/a.txt", "/b.txt")), ErrorKind::Other);
                }
            }

            mod copy_dir_all {
                use super::*;

                #[test]
                fn copy_dir_all_should_copy_tree_with_symlinks_as_symlinks() {
                    crate::fs::tests::fs::copy_dir_all::copy_dir_all_should_copy_tree_with_symlinks_as_symlinks(
                        &TestFs::new(),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn copy_dir_all_should_copy_dir_permissions() {
                    crate::fs::tests::fs::copy_dir_all::copy_dir_all_should_copy_dir_permissions(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn copy_dir_all_invalid_path_should_fail() {
                    crate::fs::tests::fs::copy_dir_all::copy_dir_all_invalid_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[cfg(windows)]
                #[test]
                fn copy_dir_all_should_keep_symlink_kind() {
                    let fs = TestFs::new();
                    fs.create_dir("/dir").unwrap();
                    fs.create_symlink_file("/dir/file", "missing").unwrap();
                    fs.create_symlink_dir("/dir/dir", "missing").unwrap();

                    fs.copy_dir_all("/dir", "/copy").unwrap();

                    assert!(!fs.symlink_metadata("/copy/file").unwrap().symlink_dir);
                    assert!(fs.symlink_metadata("/copy/dir").unwrap().symlink_dir);
                }

                #[cfg(windows)]
                #[test]
                fn copy_dir_all_symlink_metadata_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.create_dir("/dir").unwrap();
                    fs.create_symlink_dir("/dir/link", "missing").unwrap();
                    fs.set_io_error(FsOp::Metadata, "/dir/link", ErrorKind::Other);

                    assert_eq!(kind(fs.copy_dir_all("/dir", "/copy")), ErrorKind::Other);
                }

                #[test]
                fn copy_dir_all_empty_path_should_fail() {
                    let fs = TestFs::new();

                    assert_eq!(kind(fs.copy_dir_all("", "/b")), ErrorKind::NotFound);
                    assert_eq!(kind(fs.copy_dir_all("/", "")), ErrorKind::NotFound);
                }
            }

            mod remove_file {
                use super::*;

                #[test]
                fn remove_file_should_remove_file_or_symlink_only() {
                    crate::fs::tests::fs::remove_file::remove_file_should_remove_file_or_symlink_only(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn remove_file_invalid_path_should_fail() {
                    crate::fs::tests::fs::remove_file::remove_file_invalid_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn remove_file_symlink_to_dir_with_trailing_separator_should_fail() {
                    crate::fs::tests::fs::remove_file::remove_file_symlink_to_dir_with_trailing_separator_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn remove_file_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.remove_file("")), ErrorKind::NotFound);
                }

                #[test]
                fn remove_file_should_forget_removed_symlink() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "a").unwrap();
                    fs.create_symlink_file("/link", "a.txt").unwrap();

                    fs.remove_file("/link").unwrap();
                    fs.remove_file("/a.txt").unwrap();

                    assert_eq!(fs.read_dir("/").unwrap().count(), 0);
                    assert_eq!(fs.symlinks.load(Ordering::Relaxed), 0);
                }

                #[test]
                fn remove_file_borrowed_file_or_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "").unwrap();
                    let _read = fs.open_read("/a.txt").unwrap();
                    fs.set_io_error(FsOp::Remove, "/fault", ErrorKind::Other);

                    assert_eq!(
                        description(fs.remove_file("/a.txt")),
                        format!(
                            "File could not be removed, it is already borrowed for reading 1 times: {}",
                            shown("/a.txt")
                        )
                    );
                    assert_eq!(kind(fs.remove_file("/fault")), ErrorKind::Other);
                }
            }

            mod remove_dir {
                use super::*;

                #[test]
                fn remove_dir_should_remove_only_empty_dir() {
                    crate::fs::tests::fs::remove_dir::remove_dir_should_remove_only_empty_dir(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn remove_dir_invalid_path_should_fail() {
                    crate::fs::tests::fs::remove_dir::remove_dir_invalid_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[cfg(unix)]
                #[test]
                fn remove_dir_symlink_to_dir_with_trailing_separator_should_fail() {
                    crate::fs::tests::fs::remove_dir::remove_dir_symlink_to_dir_with_trailing_separator_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn remove_dir_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.remove_dir("")), ErrorKind::NotFound);
                }

                #[cfg(windows)]
                #[test]
                fn remove_dir_should_remove_dir_symlink_but_not_target() {
                    let fs = TestFs::new();
                    fs.create_dir("/dir").unwrap();
                    fs.create_symlink_dir("/link", "/dir").unwrap();

                    fs.remove_dir("/link").unwrap();

                    assert_eq!(kind(fs.symlink_metadata("/link")), ErrorKind::NotFound);
                    assert!(fs.exists("/dir").unwrap());
                }

                #[test]
                fn remove_dir_root_or_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::Remove, "/fault", ErrorKind::Other);

                    assert_eq!(kind(fs.remove_dir("/")), ErrorKind::ResourceBusy);
                    assert_eq!(kind(fs.remove_dir("/fault")), ErrorKind::Other);
                }
            }

            mod remove_dir_all {
                use super::*;

                #[test]
                fn remove_dir_all_should_remove_tree_but_not_symlink_targets() {
                    crate::fs::tests::fs::remove_dir_all::remove_dir_all_should_remove_tree_but_not_symlink_targets(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn remove_dir_all_root_should_fail_without_removing_anything() {
                    crate::fs::tests::fs::remove_dir_all::remove_dir_all_root_should_fail_without_removing_anything(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn remove_dir_all_trailing_separator_should_remove_dir() {
                    crate::fs::tests::fs::remove_dir_all::remove_dir_all_trailing_separator_should_remove_dir(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn remove_dir_all_symlink_with_trailing_separator_should_fail_without_removing_target()
                 {
                    crate::fs::tests::fs::remove_dir_all::remove_dir_all_symlink_with_trailing_separator_should_fail_without_removing_target(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn remove_dir_all_missing_path_with_trailing_separator_should_fail() {
                    crate::fs::tests::fs::remove_dir_all::remove_dir_all_missing_path_with_trailing_separator_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn remove_dir_all_invalid_path_should_fail() {
                    crate::fs::tests::fs::remove_dir_all::remove_dir_all_invalid_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn remove_dir_all_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.remove_dir_all("")), ErrorKind::NotFound);
                }

                #[test]
                fn remove_dir_all_should_forget_removed_symlinks() {
                    let fs = TestFs::new();
                    fs.create_dir_all("/dir/sub").unwrap();
                    fs.write("/b.txt", "").unwrap();
                    fs.create_symlink_file("/dir/link", "/b.txt").unwrap();
                    fs.create_symlink_file("/link", "/b.txt").unwrap();

                    fs.remove_dir_all("/dir").unwrap();
                    fs.remove_dir_all("/link").unwrap();

                    let names: Vec<_> = fs.read_dir("/").unwrap().map(|e| e.unwrap()).collect();
                    assert_eq!(names, [DirEntry::new(root("/b.txt"), FileType::File)]);
                    assert_eq!(fs.symlinks.load(Ordering::Relaxed), 0);
                }

                #[test]
                fn remove_dir_all_borrowed_file_should_remove_nothing() {
                    let fs = TestFs::new();
                    fs.create_dir_all("/dir/sub").unwrap();
                    fs.write("/dir/sub/a.txt", "").unwrap();
                    fs.create_symlink_dir("/link", "/dir").unwrap();
                    let _write = fs.open_write("/dir/sub/a.txt").unwrap();

                    assert_eq!(
                        description(fs.remove_dir_all("/link/")),
                        format!(
                            "Directory could not be removed, path is a symlink: {}",
                            shown("/link/")
                        )
                    );
                    assert_eq!(
                        description(fs.remove_dir_all("/dir")),
                        format!(
                            "File could not be removed, it is already borrowed for writing: {}",
                            shown("/dir/sub/a.txt")
                        )
                    );
                    assert!(fs.exists("/dir/sub").unwrap());
                }

                #[test]
                fn remove_dir_all_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::Remove, "/fault", ErrorKind::Other);

                    assert_eq!(kind(fs.remove_dir_all("/fault")), ErrorKind::Other);
                }
            }

            mod rename {
                use super::*;

                #[test]
                fn rename_should_replace_target_atomically() {
                    crate::fs::tests::fs::rename::rename_should_replace_target_atomically(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn rename_should_move_tree() {
                    crate::fs::tests::fs::rename::rename_should_move_tree(&TestFs::new());
                }

                #[test]
                fn rename_to_new_path_should_move_entry() {
                    crate::fs::tests::fs::rename::rename_to_new_path_should_move_entry(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn rename_same_entry_should_do_nothing() {
                    crate::fs::tests::fs::rename::rename_same_entry_should_do_nothing(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn rename_hard_link_onto_same_file_should_do_nothing() {
                    crate::fs::tests::fs::rename::rename_hard_link_onto_same_file_should_do_nothing(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn rename_invalid_source_should_fail() {
                    crate::fs::tests::fs::rename::rename_invalid_source_should_fail(&TestFs::new());
                }

                #[test]
                fn rename_invalid_target_should_fail() {
                    crate::fs::tests::fs::rename::rename_invalid_target_should_fail(&TestFs::new());
                }

                #[test]
                fn rename_target_with_separator_should_require_dir() {
                    crate::fs::tests::fs::rename::rename_target_with_separator_should_require_dir(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn rename_unresolvable_path_should_fail() {
                    let fs = looped();

                    fs.write("/a.txt", "").unwrap();
                    assert_eq!(kind(fs.rename("", "/b")), ErrorKind::NotFound);
                    assert_eq!(kind(fs.rename("/a.txt", "")), ErrorKind::NotFound);
                    assert!(fs.rename("/a.txt", "/loop/x").is_err());
                    assert!(fs.rename("/loop/x", "/b").is_err());
                    assert!(fs.rename("/a.txt", "/loop/").is_err());
                }

                #[test]
                fn rename_onto_symlink_should_forget_it() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "a").unwrap();
                    fs.create_symlink_file("/link", "/b.txt").unwrap();

                    fs.rename("/a.txt", "/link").unwrap();

                    assert_eq!(fs.symlinks.load(Ordering::Relaxed), 0);
                }

                #[test]
                fn rename_root_or_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::Rename, "/fault", ErrorKind::Other);

                    assert_eq!(kind(fs.rename("/", "/b")), ErrorKind::ResourceBusy);
                    assert_eq!(kind(fs.rename("/fault", "/b")), ErrorKind::Other);
                }

                #[test]
                fn rename_onto_borrowed_file_should_fail() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "").unwrap();
                    fs.write("/b.txt", "").unwrap();
                    let _read = fs.open_read("/b.txt").unwrap();

                    assert_eq!(
                        description(fs.rename("/a.txt", "/b.txt")),
                        format!(
                            "File could not be replaced, it is already borrowed for reading 1 times: {}",
                            shown("/b.txt")
                        )
                    );
                }
            }

            mod set_readonly {
                use super::*;

                #[cfg(unix)]
                #[test]
                fn set_readonly_should_map_to_write_bits() {
                    crate::fs::tests::fs::set_readonly::set_readonly_should_map_to_write_bits(
                        &TestFs::new(),
                    );
                }

                #[cfg(windows)]
                #[test]
                fn set_readonly_should_set_and_clear_attribute() {
                    crate::fs::tests::fs::set_readonly::set_readonly_should_set_and_clear_attribute(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn set_readonly_missing_path_should_fail() {
                    crate::fs::tests::fs::set_readonly::set_readonly_missing_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn set_readonly_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.set_readonly("", true)), ErrorKind::NotFound);
                }

                #[test]
                fn set_readonly_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::SetPermissions, "/fault", ErrorKind::Other);

                    assert_eq!(kind(fs.set_readonly("/fault", true)), ErrorKind::Other);
                }
            }

            #[cfg(unix)]
            mod set_executable {
                use super::*;

                #[test]
                fn set_executable_should_map_to_execute_bits() {
                    crate::fs::tests::fs::set_executable::set_executable_should_map_to_execute_bits(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn set_executable_dir_should_fail() {
                    crate::fs::tests::fs::set_executable::set_executable_dir_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn set_executable_missing_path_should_fail() {
                    crate::fs::tests::fs::set_executable::set_executable_missing_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn set_executable_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(kind(fs.set_executable("", true)), ErrorKind::NotFound);
                }

                #[test]
                fn set_executable_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.create_dir("/fault").unwrap();
                    fs.write("/fault.sh", "").unwrap();
                    fs.set_io_error(FsOp::SetPermissions, "/fault", ErrorKind::Other);
                    fs.set_io_error(FsOp::SetPermissions, "/fault.sh", ErrorKind::Other);

                    assert_eq!(kind(fs.set_executable("/fault", true)), ErrorKind::Other);
                    assert_eq!(kind(fs.set_executable("/fault.sh", true)), ErrorKind::Other);
                }
            }

            mod set_modified {
                use super::*;

                #[test]
                fn set_modified_should_set_modification_time() {
                    crate::fs::tests::fs::set_modified::set_modified_should_set_modification_time(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn set_modified_missing_path_should_fail() {
                    crate::fs::tests::fs::set_modified::set_modified_missing_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn set_modified_unresolvable_path_should_fail() {
                    let fs = looped();

                    assert_eq!(
                        kind(fs.set_modified("", SystemTime::UNIX_EPOCH)),
                        ErrorKind::NotFound
                    );
                }

                #[test]
                fn set_modified_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.set_io_error(FsOp::SetTimes, "/fault", ErrorKind::Other);

                    assert_eq!(
                        kind(fs.set_modified("/fault", SystemTime::UNIX_EPOCH)),
                        ErrorKind::Other
                    );
                }
            }

            #[cfg(unix)]
            mod set_mode {
                use super::*;

                #[test]
                fn set_mode_should_set_permission_bits_only() {
                    crate::fs::tests::fs::set_mode::set_mode_should_set_permission_bits_only(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn set_mode_missing_path_should_fail() {
                    crate::fs::tests::fs::set_mode::set_mode_missing_path_should_fail(
                        &TestFs::new(),
                    );
                }

                #[test]
                fn set_mode_should_keep_permission_bits() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "").unwrap();

                    fs.set_mode("/a.txt", 0o107_755).unwrap();

                    assert_eq!(fs.metadata("/a.txt").unwrap().mode, 0o7755);
                }
            }
        }
    }

    mod test_file_handle {
        use super::*;

        mod seek {
            use super::*;

            #[test]
            fn seek_invalid_position_should_fail_with_invalid_input() {
                let fs = TestFs::new();
                fs.write("/a.txt", "a").unwrap();
                let mut file = fs.open_read("/a.txt").unwrap();
                file.seek(SeekFrom::Start(1)).unwrap();

                for pos in [
                    SeekFrom::Start(u64::MAX),
                    SeekFrom::End(i64::MAX),
                    SeekFrom::Current(i64::MAX),
                    SeekFrom::End(-2),
                    SeekFrom::Current(-2),
                ] {
                    assert_eq!(
                        file.seek(pos).unwrap_err().kind(),
                        ErrorKind::InvalidInput,
                        "pos: {pos:?}"
                    );
                }

                assert_eq!(file.stream_position().unwrap(), 1);
            }

            #[test]
            fn seek_fault_should_fail() {
                let fs = TestFs::new();
                let mut file = fs.open_write("/a.txt").unwrap();
                fs.set_io_error(FsOp::Seek, "", ErrorKind::Other);

                assert_eq!(
                    file.seek(SeekFrom::End(0)).unwrap_err().kind(),
                    ErrorKind::Other
                );
            }
        }

        mod drop {
            use super::*;

            #[test]
            fn drop_should_release_borrow() {
                let fs = TestFs::new();
                fs.write("/a.txt", "").unwrap();
                let read1 = fs.open_read("/a.txt").unwrap();
                let read2 = fs.open_read("/a.txt").unwrap();

                drop(read1);
                assert_eq!(state(&fs, "/a.txt"), Some(BorrowState::Read(1)));
                drop(read2);

                assert_eq!(state(&fs, "/a.txt"), Some(BorrowState::Read(0)));
            }
        }
    }

    mod test_file_read {
        use super::*;

        mod read {
            use super::*;

            #[allow(clippy::module_inception)]
            mod read {
                use super::*;

                #[test]
                fn read_should_read_sparse_zeros() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "ab").unwrap();
                    fs.set_len("/a.txt", 4).unwrap();
                    let mut file = fs.open_read("/a.txt").unwrap();
                    let mut buf = [9; 5];

                    assert_eq!(file.read(&mut buf).unwrap(), 4);
                    assert_eq!(buf, [b'a', b'b', 0, 0, 9]);
                    assert_eq!(file.read(&mut buf).unwrap(), 0);
                }

                #[test]
                fn read_fault_should_fail() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "ab").unwrap();
                    let mut file = fs.open_read("/a.txt").unwrap();
                    fs.set_io_error(FsOp::Read, "", ErrorKind::Other);

                    assert_eq!(file.read(&mut [0; 2]).unwrap_err().kind(), ErrorKind::Other);
                    assert_eq!(
                        file.read_to_end(&mut Vec::new()).unwrap_err().kind(),
                        ErrorKind::Other
                    );
                }
            }

            mod read_to_end {
                use super::*;

                #[test]
                fn read_to_end_should_append_rest_of_sparse_file() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "content").unwrap();
                    fs.set_len("/a.txt", 9).unwrap();
                    let mut file = fs.open_read("/a.txt").unwrap();
                    file.read_exact(&mut [0; 4]).unwrap();
                    let mut buf = b"x".to_vec();

                    assert_eq!(file.read_to_end(&mut buf).unwrap(), 5);
                    assert_eq!(buf, b"xent\0\0");
                }

                #[test]
                fn read_to_end_huge_sparse_file_should_fail_with_out_of_memory() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "").unwrap();
                    fs.set_len("/a.txt", i64::MAX as u64).unwrap();
                    let mut file = fs.open_read("/a.txt").unwrap();

                    assert_eq!(
                        file.read_to_end(&mut Vec::new()).unwrap_err().kind(),
                        ErrorKind::OutOfMemory
                    );
                }
            }
        }

        mod seek {
            use super::*;

            #[test]
            fn seek_should_move_position() {
                let fs = TestFs::new();
                fs.write("/a.txt", "abc").unwrap();
                let mut file = fs.open_read("/a.txt").unwrap();

                assert_eq!(file.seek(SeekFrom::End(-1)).unwrap(), 2);
                let mut buf = String::new();
                file.read_to_string(&mut buf).unwrap();

                assert_eq!(buf, "c");
            }
        }

        mod read_file {
            use super::*;

            /// Returns a filesystem with the file `/a.txt` holding `content`
            /// and a handle reading it.
            fn content_file() -> (TestFs, TestFileRead) {
                let fs = TestFs::new();
                fs.write("/a.txt", "content").unwrap();
                let file = fs.open_read("/a.txt").unwrap();
                (fs, file)
            }

            mod path {
                use super::*;

                #[test]
                fn path_should_return_path_resolved_against_current_dir() {
                    let (fs, _file) = content_file();
                    fs.create_symlink_file("/link", "a.txt").unwrap();

                    crate::fs::tests::read_file::path::path_should_return_path_resolved_against_current_dir(
                        &fs.open_read("link").unwrap(),
                        &root("link"),
                    );
                }
            }

            mod metadata {
                use super::*;

                #[test]
                fn metadata_should_return_metadata_of_open_file() {
                    let (fs, file) = content_file();
                    crate::fs::tests::read_file::metadata::metadata_should_return_metadata_of_open_file(
                        &file,
                        &fs.metadata("/a.txt").unwrap(),
                    );
                }

                #[test]
                fn metadata_fault_should_fail() {
                    let (fs, file) = content_file();
                    fs.set_io_error(FsOp::Metadata, "", ErrorKind::Other);

                    assert_eq!(kind(file.metadata()), ErrorKind::Other);
                }
            }

            mod read_at {
                use super::*;

                #[test]
                fn read_at_should_read_at_offset_from_shared_handle() {
                    let (_fs, file) = content_file();
                    crate::fs::tests::read_file::read_at::read_at_should_read_at_offset_from_shared_handle(&file);
                }

                #[test]
                fn read_at_should_read_at_offset_and_move_position() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "hello").unwrap();
                    let mut file = fs.open_read("/a.txt").unwrap();
                    let mut buf = [0; 3];

                    assert_eq!(file.read_at(&mut buf, 1).unwrap(), 3);
                    assert_eq!(&buf, b"ell");
                    assert_eq!(file.stream_position().unwrap(), 4);
                    assert_eq!(file.read_at(&mut buf, 10).unwrap(), 0);
                }

                #[test]
                fn read_at_fault_should_keep_position() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "hello").unwrap();
                    let mut file = fs.open_read("/a.txt").unwrap();
                    fs.set_io_error(FsOp::Read, "", ErrorKind::Other);

                    assert_eq!(kind(file.read_at(&mut [0; 2], 1)), ErrorKind::Other);
                    assert_eq!(file.stream_position().unwrap(), 0);
                }
            }

            mod read_exact_at {
                use super::*;

                #[test]
                fn read_exact_at_should_fill_buffer_or_fail_at_end() {
                    let (_fs, file) = content_file();
                    crate::fs::tests::read_file::read_exact_at::read_exact_at_should_fill_buffer_or_fail_at_end(&file);
                }

                #[test]
                fn read_exact_at_should_loop_over_short_and_interrupted_reads() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "hello").unwrap();
                    let file = fs.open_read("/a.txt").unwrap();
                    fs.set_short_io(FsOp::Read, "", 2);
                    fs.set_io_errors(FsOp::Read, "/a.txt", [Some(ErrorKind::Interrupted), None]);
                    let mut buf = [0; 4];

                    file.read_exact_at(&mut buf, 1).unwrap();

                    assert_eq!(&buf, b"ello");
                }

                #[test]
                fn read_exact_at_should_loop_over_short_reads() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "hello").unwrap();
                    let file = fs.open_read("/a.txt").unwrap();
                    fs.set_short_io(FsOp::Read, "", 2);
                    let mut buf = [0; 5];

                    file.read_exact_at(&mut buf, 0).unwrap();

                    assert_eq!(&buf, b"hello");
                }

                #[test]
                fn read_exact_at_error_should_fail() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "hello").unwrap();
                    let file = fs.open_read("/a.txt").unwrap();
                    fs.set_io_error(FsOp::Read, "", ErrorKind::Other);

                    assert_eq!(kind(file.read_exact_at(&mut [0; 4], 0)), ErrorKind::Other);
                }
            }

            mod lock_shared {
                use super::*;

                #[test]
                fn lock_shared_should_lock_until_unlocked() {
                    let (_fs, file) = content_file();
                    crate::fs::tests::read_file::lock_shared::lock_shared_should_lock_until_unlocked(&file);
                }

                #[test]
                fn lock_shared_should_succeed_for_every_read_handle() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "").unwrap();
                    let file = fs.open_read("/a.txt").unwrap();
                    let other = fs.open_read("/a.txt").unwrap();

                    file.lock_shared().unwrap();
                    other.lock_shared().unwrap();
                    fs.set_io_error(FsOp::Lock, "", ErrorKind::Unsupported);
                    assert_eq!(kind(file.lock_shared()), ErrorKind::Unsupported);
                }
            }

            mod try_lock_shared {
                use super::*;

                #[test]
                fn try_lock_shared_should_lock_unlocked_file() {
                    let (_fs, file) = content_file();
                    crate::fs::tests::read_file::try_lock_shared::try_lock_shared_should_lock_unlocked_file(&file);
                }
            }

            mod unlock {
                use super::*;

                #[test]
                fn unlock_should_release_lock() {
                    let (_fs, file) = content_file();
                    crate::fs::tests::read_file::unlock::unlock_should_release_lock(&file);
                }
            }
        }
    }

    mod test_file_write {
        use super::*;

        mod read {
            use super::*;

            #[allow(clippy::module_inception)]
            mod read {
                use super::*;

                #[test]
                fn read_write_only_should_fail() {
                    let fs = TestFs::new();
                    let mut file = fs
                        .open_write_options("/a.txt", OpenWriteOptions::new().create().write_only())
                        .unwrap();

                    assert!(file.read(&mut [0; 1]).is_err());
                    assert!(file.read_to_end(&mut Vec::new()).is_err());
                    assert!(file.read_at(&mut [0; 1], 0).is_err());
                }

                #[test]
                fn read_should_read_written_content() {
                    let fs = TestFs::new();
                    let mut file = fs.open_write("/a.txt").unwrap();
                    file.write_all(b"ab").unwrap();
                    file.rewind().unwrap();
                    let mut buf = [0; 2];

                    file.read_exact(&mut buf).unwrap();

                    assert_eq!(&buf, b"ab");
                }
            }
        }

        mod write {
            use super::*;

            mod write_vectored {
                use super::*;

                #[test]
                fn write_vectored_should_write_buffers_as_one_write() {
                    let fs = TestFs::new();
                    let mut file = fs.open_write("/a.txt").unwrap();
                    fs.set_short_io(FsOp::Write, "", 4);

                    let written = file
                        .write_vectored(&[IoSlice::new(b"con"), IoSlice::new(b"tent")])
                        .unwrap();

                    assert_eq!(written, 4);
                    drop(file);
                    assert_eq!(fs.read_to_string("/a.txt").unwrap(), "cont");
                }
            }

            #[allow(clippy::module_inception)]
            mod write {
                use super::*;

                #[test]
                fn write_beyond_max_file_len_should_fail() {
                    let fs = TestFs::new();
                    let mut file = fs.open_write("/a.txt").unwrap();
                    file.seek(SeekFrom::Start(MAX_FILE_LEN)).unwrap();

                    assert_eq!(
                        file.write(b"a").unwrap_err().kind(),
                        ErrorKind::FileTooLarge
                    );
                    assert_eq!(file.write(b"").unwrap(), 0);
                }

                #[test]
                fn write_append_should_write_at_end() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "ab").unwrap();
                    let mut file = fs
                        .open_write_options("/a.txt", OpenWriteOptions::new().append())
                        .unwrap();

                    file.write_all(b"c").unwrap();
                    file.write_at(b"d", 0).unwrap();
                    drop(file);

                    assert_eq!(fs.read_to_string("/a.txt").unwrap(), "abcd");
                }
            }

            mod flush {
                use super::*;

                #[test]
                fn flush_should_fail_on_fault_only() {
                    let fs = TestFs::new();
                    let mut file = fs.open_write("/a.txt").unwrap();

                    file.flush().unwrap();
                    fs.set_io_error(FsOp::Flush, "", ErrorKind::Other);
                    assert_eq!(file.flush().unwrap_err().kind(), ErrorKind::Other);
                }
            }
        }

        mod read_file {
            use super::*;

            /// Returns a filesystem with the file `/a.txt` holding `content`
            /// and a handle reading and writing it.
            fn content_file() -> (TestFs, TestFileWrite) {
                let fs = TestFs::new();
                fs.write("/a.txt", "content").unwrap();
                let file = fs.open_write("/a.txt").unwrap();
                (fs, file)
            }

            mod path {
                use super::*;

                #[test]
                fn path_should_return_path_resolved_against_current_dir() {
                    let (fs, _file) = content_file();
                    fs.create_symlink_file("/link", "b.txt").unwrap();

                    crate::fs::tests::read_file::path::path_should_return_path_resolved_against_current_dir(
                        &fs.open_write("link").unwrap(),
                        &root("link"),
                    );
                }
            }

            mod metadata {
                use super::*;

                #[test]
                fn metadata_should_return_metadata_of_open_file() {
                    let (fs, file) = content_file();
                    crate::fs::tests::read_file::metadata::metadata_should_return_metadata_of_open_file(
                        &file,
                        &fs.metadata("/a.txt").unwrap(),
                    );
                }
            }

            mod read_at {
                use super::*;

                #[test]
                fn read_at_should_read_at_offset_from_shared_handle() {
                    let (_fs, file) = content_file();
                    crate::fs::tests::read_file::read_at::read_at_should_read_at_offset_from_shared_handle(&file);
                }

                #[test]
                fn read_at_should_read_and_move_position() {
                    let fs = TestFs::new();
                    fs.write("/a.txt", "hello").unwrap();
                    let mut file = fs.open_write("/a.txt").unwrap();
                    let mut buf = [0; 2];

                    assert_eq!(file.read_at(&mut buf, 3).unwrap(), 2);
                    assert_eq!(&buf, b"lo");
                    assert_eq!(file.stream_position().unwrap(), 5);
                    fs.set_io_error(FsOp::Read, "", ErrorKind::Other);
                    assert!(file.read_at(&mut buf, 0).is_err());
                }
            }

            mod read_exact_at {
                use super::*;

                #[test]
                fn read_exact_at_should_fill_buffer_or_fail_at_end() {
                    let (_fs, file) = content_file();
                    crate::fs::tests::read_file::read_exact_at::read_exact_at_should_fill_buffer_or_fail_at_end(&file);
                }
            }

            mod lock_shared {
                use super::*;

                #[test]
                fn lock_shared_should_lock_until_unlocked() {
                    let (_fs, file) = content_file();
                    crate::fs::tests::read_file::lock_shared::lock_shared_should_lock_until_unlocked(&file);
                }
            }

            mod try_lock_shared {
                use super::*;

                #[test]
                fn try_lock_shared_should_lock_unlocked_file() {
                    let (_fs, file) = content_file();
                    crate::fs::tests::read_file::try_lock_shared::try_lock_shared_should_lock_unlocked_file(&file);
                }
            }

            mod unlock {
                use super::*;

                #[test]
                fn unlock_should_release_lock() {
                    let (_fs, file) = content_file();
                    crate::fs::tests::read_file::unlock::unlock_should_release_lock(&file);
                }
            }
        }

        mod write_file {
            use super::*;

            /// Returns a filesystem with the file `/a.txt` holding `content`
            /// and a handle opened on it with `options`.
            fn content_file(options: OpenWriteOptions) -> (TestFs, TestFileWrite) {
                let fs = TestFs::new();
                fs.write("/a.txt", "content").unwrap();
                let file = fs.open_write_options("/a.txt", options).unwrap();
                (fs, file)
            }

            mod write_at {
                use super::*;

                #[test]
                fn write_at_should_write_at_offset_and_extend_file() {
                    let (_fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::write_at::write_at_should_write_at_offset_and_extend_file(&file);
                }

                #[cfg(target_os = "linux")]
                #[test]
                fn write_at_append_should_ignore_offset() {
                    let (_fs, file) = content_file(OpenWriteOptions::new().append());
                    crate::fs::tests::write_file::write_at::write_at_append_should_ignore_offset(
                        &file,
                    );
                }

                #[test]
                fn write_at_should_extend_file_and_move_position() {
                    let fs = TestFs::new();
                    let mut file = fs.open_write("/a.txt").unwrap();

                    assert_eq!(file.write_at(b"b", 1).unwrap(), 1);
                    assert_eq!(file.stream_position().unwrap(), 2);
                    file.write_at(b"a", 0).unwrap();
                    drop(file);

                    assert_eq!(fs.read_to_string("/a.txt").unwrap(), "ab");
                }

                #[test]
                fn write_at_append_from_threads_should_not_overwrite_each_other() {
                    let fs = TestFs::new();
                    fs.set_capacity(Some(1000));
                    let file = fs
                        .open_write_options("/a.txt", OpenWriteOptions::new().create().append())
                        .unwrap();

                    std::thread::scope(|scope| {
                        for byte in *b"ab" {
                            let file = &file;
                            scope.spawn(move || {
                                for _ in 0..100 {
                                    file.write_at(&[byte], 0).unwrap();
                                }
                            });
                        }
                    });
                    drop(file);

                    let data = fs.read("/a.txt").unwrap();
                    assert_eq!(data.len(), 200);
                    assert_eq!(data.iter().filter(|&&byte| byte == b'a').count(), 100);
                }
            }

            mod write_all_at {
                use super::*;

                #[test]
                fn write_all_at_should_write_whole_buffer_at_offset() {
                    let (_fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::write_all_at::write_all_at_should_write_whole_buffer_at_offset(&file);
                }

                #[test]
                fn write_all_at_should_loop_over_short_and_interrupted_writes() {
                    let fs = TestFs::new();
                    let file = fs.open_write("/a.txt").unwrap();
                    fs.set_short_io(FsOp::Write, "", 2);
                    fs.set_io_errors(FsOp::Write, "/a.txt", [Some(ErrorKind::Interrupted), None]);

                    file.write_all_at(b"hello", 0).unwrap();
                    drop(file);

                    assert_eq!(fs.read_to_string("/a.txt").unwrap(), "hello");
                }

                #[test]
                fn write_all_at_should_loop_over_short_writes() {
                    let fs = TestFs::new();
                    let file = fs.open_write("/a.txt").unwrap();
                    fs.set_short_io(FsOp::Write, "", 2);

                    file.write_all_at(b"hello", 1).unwrap();
                    drop(file);

                    assert_eq!(fs.read("/a.txt").unwrap(), b"\0hello");
                }

                #[test]
                fn write_all_at_zero_bytes_written_should_fail_with_write_zero() {
                    let fs = TestFs::new();
                    let file = fs.open_write("/a.txt").unwrap();
                    fs.set_short_io(FsOp::Write, "", 0);

                    assert_eq!(
                        description(file.write_all_at(b"a", 0)),
                        format!(
                            "File could not be written, no bytes were written: {}",
                            shown("/a.txt")
                        )
                    );
                    assert_eq!(kind(file.write_all_at(b"a", 0)), ErrorKind::WriteZero);
                }

                #[test]
                fn write_all_at_error_should_fail() {
                    let fs = TestFs::new();
                    let file = fs.open_write("/a.txt").unwrap();
                    fs.set_io_error(FsOp::Write, "", ErrorKind::StorageFull);

                    assert_eq!(kind(file.write_all_at(b"a", 0)), ErrorKind::StorageFull);
                }
            }

            mod write_all_slices {
                use super::*;

                #[test]
                fn write_all_slices_should_write_all_buffers_at_position() {
                    let (_fs, mut file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::write_all_slices::write_all_slices_should_write_all_buffers_at_position(&mut file);
                }

                #[test]
                fn write_all_slices_append_should_write_at_end() {
                    let (_fs, mut file) = content_file(OpenWriteOptions::new().append());
                    crate::fs::tests::write_file::write_all_slices::write_all_slices_append_should_write_at_end(&mut file);
                }

                #[test]
                fn write_all_slices_should_loop_over_short_and_interrupted_writes() {
                    let fs = TestFs::new();
                    let mut file = fs.open_write("/a.txt").unwrap();
                    fs.set_short_io(FsOp::Write, "", 2);
                    fs.set_io_errors(FsOp::Write, "/a.txt", [Some(ErrorKind::Interrupted), None]);

                    file.write_all_slices(&mut [IoSlice::new(b"hel"), IoSlice::new(b"lo")])
                        .unwrap();
                    drop(file);

                    assert_eq!(fs.read_to_string("/a.txt").unwrap(), "hello");
                }

                #[test]
                fn write_all_slices_zero_bytes_written_should_fail_with_write_zero() {
                    let fs = TestFs::new();
                    let mut file = fs.open_write("/a.txt").unwrap();
                    fs.set_short_io(FsOp::Write, "", 0);

                    let error = file.write_all_slices(&mut [IoSlice::new(b"a")]);

                    assert_eq!(
                        description(error),
                        format!(
                            "File could not be written, no bytes were written: {}",
                            shown("/a.txt")
                        )
                    );
                    assert_eq!(
                        kind(file.write_all_slices(&mut [IoSlice::new(b"a")])),
                        ErrorKind::WriteZero
                    );
                }

                #[test]
                fn write_all_slices_error_should_fail() {
                    let fs = TestFs::new();
                    let mut file = fs.open_write("/a.txt").unwrap();
                    fs.set_io_error(FsOp::Write, "", ErrorKind::StorageFull);

                    let error = file.write_all_slices(&mut [IoSlice::new(b"a")]);

                    assert_eq!(kind(error), ErrorKind::StorageFull);
                }
            }

            mod set_len {
                use super::*;

                #[test]
                fn set_len_should_truncate_or_extend_with_zeros() {
                    let (_fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::set_len::set_len_should_truncate_or_extend_with_zeros(&file);
                }

                #[test]
                fn set_len_should_keep_position() {
                    let (_fs, mut file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::set_len::set_len_should_keep_position(&mut file);
                }

                #[test]
                fn set_len_invalid_length_should_fail() {
                    let (_fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::set_len::set_len_invalid_length_should_fail(
                        &file,
                    );
                }

                #[test]
                fn set_len_fault_should_fail() {
                    let (fs, file) = content_file(OpenWriteOptions::new());
                    fs.set_io_error(FsOp::SetLen, "", ErrorKind::Other);

                    assert_eq!(kind(file.set_len(1)), ErrorKind::Other);
                }
            }

            mod sync_all {
                use super::*;

                #[test]
                fn sync_all_should_succeed() {
                    let (_fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::sync_all::sync_all_should_succeed(&file);
                }

                #[test]
                fn sync_all_should_fail_on_fault_only() {
                    let fs = TestFs::new();
                    let file = fs.open_write("/a.txt").unwrap();

                    file.sync_all().unwrap();
                    fs.set_io_error(FsOp::SyncAll, "", ErrorKind::Other);
                    assert_eq!(kind(file.sync_all()), ErrorKind::Other);
                }
            }

            mod sync_data {
                use super::*;

                #[test]
                fn sync_data_should_succeed() {
                    let (_fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::sync_data::sync_data_should_succeed(&file);
                }

                #[test]
                fn sync_data_should_fail_on_fault_only() {
                    let fs = TestFs::new();
                    let file = fs.open_write("/a.txt").unwrap();

                    file.sync_data().unwrap();
                    fs.set_io_error(FsOp::SyncData, "", ErrorKind::Other);
                    assert_eq!(kind(file.sync_data()), ErrorKind::Other);
                }
            }

            mod lock {
                use super::*;

                #[test]
                fn lock_should_lock_until_unlocked() {
                    let (_fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::lock::lock_should_lock_until_unlocked(&file);
                }
            }

            mod try_lock {
                use super::*;

                #[test]
                fn try_lock_should_lock_unlocked_file() {
                    let (_fs, file) = content_file(OpenWriteOptions::new());
                    crate::fs::tests::write_file::try_lock::try_lock_should_lock_unlocked_file(
                        &file,
                    );
                }
            }
        }
    }

    mod test_read_dir {
        use super::*;

        mod iterator {
            use super::*;

            #[test]
            fn iterator_should_see_changes_and_end_when_dir_removed() {
                let fs = TestFs::new();
                fs.create_dir("/dir").unwrap();
                fs.write("/dir/a", "").unwrap();
                let mut entries = fs.read_dir("/dir").unwrap();

                assert_eq!(entries.next().unwrap().unwrap().file_name(), "a");
                assert!(entries.next().is_none());
                fs.remove_file("/dir/a").unwrap();
                fs.remove_dir("/dir").unwrap();
                assert!(entries.next().is_none());
            }

            #[test]
            fn iterator_entry_fault_should_fail_entry_and_continue() {
                let fs = TestFs::new();
                fs.create_dir("/dir").unwrap();
                fs.write("/dir/a", "").unwrap();
                fs.write("/dir/b", "").unwrap();
                fs.set_io_error(FsOp::ReadDirEntry, "/dir/a", ErrorKind::Other);

                let entries: Vec<_> = fs.read_dir("/dir").unwrap().collect();

                assert_eq!(entries.len(), 2);
                let failed = entries
                    .iter()
                    .find_map(|entry| entry.as_ref().err())
                    .unwrap();
                assert!(
                    failed
                        .description()
                        .replace('\\', "/")
                        .ends_with(&format!(": {}", shown("/dir/a")))
                );
                assert!(
                    entries
                        .iter()
                        .any(|entry| entry.as_ref().is_ok_and(|entry| entry.file_name() == "b"))
                );
            }
        }
    }

    mod insert_entry {
        use super::*;

        #[test]
        fn insert_entry_without_parent_dir_should_insert_entry_only() {
            let mut tree = Tree::new();

            insert_entry(
                &mut tree,
                PathBuf::from(ROOT),
                TestFsEntry::Dir(TestFsDir::new(SystemTime::now())),
            );

            assert!(is_dir(&tree, Path::new(ROOT)));
        }
    }

    mod lock_file_read {
        use super::*;

        #[test]
        fn lock_file_read_poisoned_should_recover() {
            let file = RwLock::new(TestFsFile::new(SystemTime::now(), new_dir_attributes()));
            std::thread::scope(|scope| {
                scope
                    .spawn(|| {
                        let _guard = file.write().unwrap();
                        panic!("poison");
                    })
                    .join()
                    .unwrap_err();
            });

            assert_eq!(lock_file_read(&file).len, 0);
            assert!(!file.is_poisoned());
        }
    }

    mod lock_file_write {
        use super::*;

        #[test]
        fn lock_file_write_poisoned_should_recover() {
            let file = RwLock::new(TestFsFile::new(SystemTime::now(), new_dir_attributes()));
            std::thread::scope(|scope| {
                scope
                    .spawn(|| {
                        let _guard = file.write().unwrap();
                        panic!("poison");
                    })
                    .join()
                    .unwrap_err();
            });

            assert_eq!(lock_file_write(&file).len, 0);
            assert!(!file.is_poisoned());
        }
    }
}
