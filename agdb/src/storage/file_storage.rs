use super::StorageData;
use super::StorageSlice;
use super::SyncMode;
use super::write_ahead_log::WriteAheadLog;
use super::write_ahead_log::remove_wal;
use super::write_ahead_log::wal_filename;
use crate::DbError;
use crate::DbErrorType;
use agplatform::Fs;
use agplatform::OpenWriteOptions;
use agplatform::Platform;
use agplatform::PlatformImpl;
use agplatform::ReadFile;
use agplatform::WriteFile;
use std::io::Seek;
use std::io::SeekFrom;
use std::sync::Arc;

type FileHandle<P> = <<P as Platform>::Fs as Fs>::WriteHandle;

pub(crate) type OldBytes<'a, P> =
    dyn Fn(&FileHandle<P>, u64, &mut [u8]) -> Result<(), DbError> + 'a;

/// Single file based storage with write ahead log (WAL) for resiliency
/// implementing [`StorageData`]. It uses the storage name as the file
/// name (`.{name}` for WAL). The files are accessed through the platform
/// `P`, see [`FileStorage`] for the production one. Multiple readers
/// share the single file handle reading it positionally. The
/// [`StorageData::read()`] always returns owning buffer.
///
/// The [`StorageData::backup()`] is implemented so that it copies the file
/// to the new name.
///
/// The [`StorageData::flush()`] clears the WAL. When `sync_mode` is
/// [`SyncMode::Commit`], flush also issues `fdatasync` on the main file
/// before clearing the WAL (ensuring committed data is durable) and on
/// the WAL file after clearing (ensuring the clear is durable). This
/// prevents a crash from rolling back an already-committed transaction.
/// Recovering from the WAL always syncs the restored file before clearing
/// the WAL.
pub struct FileStorageImpl<P: Platform> {
    platform: Arc<P>,
    file: FileHandle<P>,
    filename: String,
    len: u64,
    sync_mode: SyncMode,
    wal: WriteAheadLog<P>,
}

/// The [`FileStorageImpl`] on the production [`PlatformImpl`].
pub type FileStorage = FileStorageImpl<PlatformImpl>;

impl<P: Platform> std::fmt::Debug for FileStorageImpl<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileStorage")
            .field("filename", &self.filename)
            .field("len", &self.len)
            .field("sync_mode", &self.sync_mode)
            .field("wal", &self.wal)
            .finish_non_exhaustive()
    }
}

impl<P: Platform> FileStorageImpl<P> {
    /// Constructs or loads the storage `name` in the file system of the
    /// `platform`, see [`StorageData::new()`].
    pub fn with_platform(platform: Arc<P>, name: &str) -> Result<Self, DbError> {
        let mut file = platform
            .fs()
            .open_write_options(name, OpenWriteOptions::new().create())?;
        let mut wal = WriteAheadLog::new(&*platform, name)?;
        let sync_mode = SyncMode::None;

        Self::apply_wal(&file, &mut wal)?;

        let len = file.seek(SeekFrom::End(0))?;

        Ok(Self {
            platform,
            file,
            filename: name.to_string(),
            len,
            sync_mode,
            wal,
        })
    }

    pub(crate) fn platform(&self) -> &Arc<P> {
        &self.platform
    }

    pub(crate) fn read_all(&self) -> Result<Vec<u8>, DbError> {
        let mut buffer = vec![0_u8; self.len as usize];
        self.file.read_exact_at(&mut buffer, 0)?;
        Ok(buffer)
    }

    pub(crate) fn resize_with(
        &mut self,
        new_len: u64,
        old_bytes: &OldBytes<P>,
    ) -> Result<(), DbError> {
        let current_len = self.len;

        if new_len < current_len {
            self.wal
                .insert_with(new_len, current_len - new_len, &|buffer| {
                    old_bytes(&self.file, new_len, buffer)
                })?;
        } else if current_len < new_len {
            self.wal.insert(current_len, &[])?;
        } else {
            return Ok(());
        }

        self.file.set_len(new_len)?;
        self.len = new_len;
        Ok(())
    }

    pub(crate) fn write_with(
        &mut self,
        pos: u64,
        bytes: &[u8],
        old_bytes: &OldBytes<P>,
    ) -> Result<(), DbError> {
        if bytes.is_empty() {
            return Ok(());
        }

        let current_len = self.len;
        let end = pos + bytes.len() as u64;

        if pos < current_len {
            self.wal
                .insert_with(pos, std::cmp::min(current_len, end) - pos, &|buffer| {
                    old_bytes(&self.file, pos, buffer)
                })?;
        }

        if current_len < end {
            self.wal.insert(current_len, &[])?;
        }

        self.file.write_all_at(bytes, pos)?;
        self.len = std::cmp::max(current_len, end);
        Ok(())
    }

    fn apply_wal_record(file: &FileHandle<P>, pos: u64, value: &[u8]) -> Result<(), DbError> {
        if value.is_empty() {
            file.set_len(pos)?;
        } else {
            file.write_all_at(value, pos)?;
        }

        Ok(())
    }

    fn apply_wal(file: &FileHandle<P>, wal: &mut WriteAheadLog<P>) -> Result<(), DbError> {
        if wal.is_empty() {
            return Ok(());
        }

        wal.rollback(|pos, value| Self::apply_wal_record(file, pos, value))?;
        file.sync_data()?;
        wal.truncate()?;
        wal.sync()
    }

    fn read_file(file: &FileHandle<P>, pos: u64, buffer: &mut [u8]) -> Result<(), DbError> {
        Ok(file.read_exact_at(buffer, pos)?)
    }
}

impl<P: Platform + Default> StorageData for FileStorageImpl<P> {
    /// The file is copied through its handle, which keeps the borrow of a
    /// file open for writing, and the log of a file previously at `name`
    /// is removed so that it is not replayed onto the copy.
    fn backup(&self, name: &str) -> Result<(), DbError> {
        let fs = self.platform.fs();

        if fs.exists(name)? && fs.canonicalize(name)? == fs.canonicalize(&self.filename)? {
            return Err(DbError::storage(
                DbErrorType::NotAllowed,
                format!("Cannot back up '{}' onto itself", self.filename),
            ));
        }

        remove_wal(&*self.platform, name)?;
        fs.copy_from(&self.file, name)?;
        Ok(())
    }

    fn copy(&self, name: &str) -> Result<Self, DbError> {
        self.backup(name)?;
        Self::with_platform(self.platform.clone(), name)
    }

    fn flush(&mut self) -> Result<(), DbError> {
        if self.wal.is_empty() {
            return Ok(());
        }

        match self.sync_mode {
            SyncMode::None => self.wal.clear(),
            SyncMode::Commit => {
                self.file.sync_data()?;
                self.wal.truncate()?;
                self.wal.sync()
            }
        }
    }

    fn set_sync_mode(&mut self, mode: SyncMode) {
        self.sync_mode = mode;
    }

    fn sync_mode(&self) -> SyncMode {
        self.sync_mode
    }

    fn sync(&mut self) -> Result<(), DbError> {
        Ok(self.file.sync_data()?)
    }

    fn len(&self) -> u64 {
        self.len
    }

    fn name(&self) -> &str {
        &self.filename
    }

    fn new(name: &str) -> Result<Self, DbError> {
        Self::with_platform(Arc::new(P::default()), name)
    }

    fn read(&'_ self, pos: u64, value_len: u64) -> Result<StorageSlice<'_>, DbError> {
        let mut buffer = vec![0_u8; value_len as usize];
        self.file.read_exact_at(&mut buffer, pos)?;
        Ok(StorageSlice::Owned(buffer))
    }

    /// The open files are renamed with their handles kept, so that they
    /// stay valid if the renaming fails half way.
    fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
        let fs = self.platform.fs();
        fs.rename(&self.filename, new_name)?;

        if let Err(error) = fs.rename(wal_filename(&self.filename), wal_filename(new_name)) {
            let _ = fs.rename(new_name, &self.filename);
            return Err(error.into());
        }

        self.filename = new_name.to_string();
        Ok(())
    }

    fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
        self.resize_with(new_len, &Self::read_file)
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        self.write_with(pos, bytes, &Self::read_file)
    }

    fn recover_from_wal(&mut self) -> Result<(), DbError> {
        Self::apply_wal(&self.file, &mut self.wal)?;
        self.len = self.file.seek(SeekFrom::End(0))?;
        Ok(())
    }
}

impl<P: Platform> Drop for FileStorageImpl<P> {
    fn drop(&mut self) {
        let _ = Self::apply_wal(&self.file, &mut self.wal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::io::ErrorKind;

    const NAME: &str = "/db";
    const WAL: &str = "/.db";

    type TestFileStorage = FileStorageImpl<TestPlatform>;

    fn platform() -> Arc<TestPlatform> {
        Arc::new(test_platform())
    }

    fn open(platform: &Arc<TestPlatform>) -> TestFileStorage {
        FileStorageImpl::with_platform(platform.clone(), NAME).unwrap()
    }

    fn kill(platform: &Arc<TestPlatform>) {
        platform.fs().sync();
        platform.fs().crash();
    }

    fn read(storage: &TestFileStorage) -> Vec<u8> {
        storage.read_all().unwrap()
    }

    /// Leaves the log of the storage `name` holding the pre-image of an
    /// unflushed write as a process killed while writing it does.
    fn stale_storage(platform: &Arc<TestPlatform>, name: &str) {
        let mut storage = FileStorageImpl::with_platform(platform.clone(), name).unwrap();
        storage.write(0, &[9, 9, 9, 9]).unwrap();
        storage.flush().unwrap();
        storage.write(0, &[8]).unwrap();
        kill(platform);
    }

    fn written(platform: &Arc<TestPlatform>) -> TestFileStorage {
        let mut storage = open(platform);
        storage.write(0, &[1, 2, 3]).unwrap();
        storage.flush().unwrap();
        storage
    }

    mod file_storage_impl {
        use super::*;

        #[test]
        fn file_storage_impl_should_roll_back_unflushed_writes_on_reopen() {
            for killed in [false, true] {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();
                storage.write(1, &[4, 5, 6]).unwrap();
                storage.resize(2).unwrap();

                if killed {
                    kill(&platform);
                } else {
                    drop(storage);
                }

                assert_eq!(read(&open(&platform)), vec![1, 2, 3], "killed: {killed}");
            }
        }

        mod std_fmt_debug {
            use super::*;

            #[test]
            fn fmt_should_show_name_len_sync_mode_and_wal() {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(0, &[1]).unwrap();

                assert_eq!(
                    format!("{storage:?}"),
                    "FileStorage { filename: \"/db\", len: 1, sync_mode: None, wal: WriteAheadLog { generation: Some(4611686018427387904), len: 48, .. }, .. }"
                );
            }
        }

        mod with_platform {
            use super::*;

            #[test]
            fn with_platform_should_load_file() {
                let platform = platform();
                platform.fs().write(NAME, [1, 2, 3]).unwrap();

                let storage = open(&platform);

                assert_eq!(storage.len(), 3);
                assert_eq!(read(&storage), vec![1, 2, 3]);
            }

            #[test]
            fn with_platform_should_roll_back_writes_logged_before_crash() {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();
                storage.write(2, &[4, 5]).unwrap();
                kill(&platform);

                let storage = open(&platform);

                assert_eq!(read(&storage), vec![1, 2, 3]);
                assert!(storage.wal.is_empty());
            }

            #[test]
            fn with_platform_should_roll_back_writes_logged_in_legacy_log() {
                // GIVEN a write of [9, 9, 9] at 2 into [1, 2, 3, 4] logged by a version without the log header
                let platform = platform();
                platform.fs().write(NAME, [1, 2, 9, 9, 9]).unwrap();
                let legacy_record = |pos: u64, value: &[u8]| {
                    [
                        &pos.to_le_bytes()[..],
                        &(value.len() as u64).to_le_bytes(),
                        value,
                    ]
                    .concat()
                };
                platform
                    .fs()
                    .write(
                        WAL,
                        [legacy_record(2, &[3, 4]), legacy_record(4, &[])].concat(),
                    )
                    .unwrap();

                // WHEN the storage is opened
                let storage = open(&platform);

                // THEN the write is rolled back and the log is emptied
                assert_eq!(read(&storage), vec![1, 2, 3, 4]);
                assert!(storage.wal.is_empty());
                drop(storage);
                assert_eq!(platform.fs().read(NAME).unwrap(), [1, 2, 3, 4]);
                assert!(platform.fs().read(WAL).unwrap().is_empty());
            }

            #[test]
            fn with_platform_failure_should_fail() {
                for (op, path) in [
                    (FsOp::Open, NAME),
                    (FsOp::Open, WAL),
                    (FsOp::Read, WAL),
                    (FsOp::Write, NAME),
                    (FsOp::SetLen, NAME),
                    (FsOp::SyncData, NAME),
                    (FsOp::SetLen, WAL),
                    (FsOp::SyncData, WAL),
                    (FsOp::Seek, NAME),
                ] {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2, 3]).unwrap();
                    storage.flush().unwrap();
                    storage.write(2, &[4, 5]).unwrap();
                    kill(&platform);
                    platform.fs().set_io_error(op, path, ErrorKind::Other);

                    assert!(
                        FileStorageImpl::with_platform(platform.clone(), NAME).is_err(),
                        "op: {op:?}, path: {path}"
                    );
                }
            }
        }

        mod platform {
            use super::*;

            #[test]
            fn platform_should_return_platform_of_storage() {
                let platform = platform();
                let storage = open(&platform);

                assert!(Arc::ptr_eq(storage.platform(), &platform));
            }
        }

        mod read_all {
            use super::*;

            #[test]
            fn read_all_should_return_all_bytes() {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(1, &[1, 2]).unwrap();

                assert_eq!(storage.read_all(), Ok(vec![0, 1, 2]));
            }

            #[test]
            fn read_all_failure_should_fail() {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(0, &[1]).unwrap();
                platform
                    .fs()
                    .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                assert!(storage.read_all().is_err());
            }
        }

        mod resize_with {
            use super::*;

            #[test]
            fn resize_with_should_log_old_bytes_it_is_given() {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();

                storage
                    .resize_with(1, &|_, _, buffer| {
                        buffer.fill(9);
                        Ok(())
                    })
                    .unwrap();

                storage.recover_from_wal().unwrap();
                assert_eq!(read(&storage), vec![1, 9, 9]);
            }

            #[test]
            fn resize_with_old_bytes_failure_should_fail_without_resizing() {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(0, &[1, 2, 3]).unwrap();

                let result = storage.resize_with(1, &|_, _, _| {
                    Err(DbError::storage(crate::DbErrorType::NotFound, "old bytes"))
                });

                assert!(result.is_err());
                assert_eq!(read(&storage), vec![1, 2, 3]);
            }
        }

        mod write_with {
            use super::*;

            #[test]
            fn write_with_should_log_old_bytes_it_is_given() {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();

                storage
                    .write_with(1, &[4, 5, 6], &|_, pos, buffer| {
                        assert_eq!((pos, buffer.len()), (1, 2));
                        buffer.fill(9);
                        Ok(())
                    })
                    .unwrap();

                storage.recover_from_wal().unwrap();
                assert_eq!(read(&storage), vec![1, 9, 9]);
            }

            #[test]
            fn write_with_old_bytes_failure_should_fail_without_writing() {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(0, &[1, 2, 3]).unwrap();

                let result = storage.write_with(0, &[4], &|_, _, _| {
                    Err(DbError::storage(crate::DbErrorType::NotFound, "old bytes"))
                });

                assert!(result.is_err());
                assert_eq!(read(&storage), vec![1, 2, 3]);
            }
        }

        mod storage_data {
            use super::*;

            mod backup {
                use super::*;

                #[test]
                fn backup_should_write_bytes_to_name() {
                    let platform = platform();

                    crate::storage::tests::storage_data::backup::backup_should_write_bytes_to_name(
                        open(&platform),
                        "/backup",
                        |name| platform.fs().read(name).unwrap(),
                    );
                }

                #[test]
                fn backup_should_not_replay_log_of_storage_previously_at_name() {
                    // GIVEN a storage killed with an unflushed write at the name
                    let platform = platform();
                    stale_storage(&platform, "/backup");
                    let storage = written(&platform);

                    // WHEN the storage is backed up to the name
                    storage.backup("/backup").unwrap();

                    // THEN the backup opens with the bytes of the storage
                    assert!(!platform.fs().exists("/.backup").unwrap());
                    let backup =
                        FileStorageImpl::with_platform(platform.clone(), "/backup").unwrap();
                    assert_eq!(read(&backup), vec![1, 2, 3]);
                }

                #[test]
                fn backup_onto_itself_should_fail_keeping_bytes() {
                    let platform = platform();
                    let storage = written(&platform);
                    platform.fs().create_symlink_file("/link", NAME).unwrap();

                    for name in [NAME, "/link"] {
                        assert_eq!(
                            storage.backup(name),
                            Err(DbError::storage(
                                DbErrorType::NotAllowed,
                                "Cannot back up '/db' onto itself"
                            )),
                            "name: {name}"
                        );
                        assert_eq!(read(&storage), vec![1, 2, 3], "name: {name}");
                    }
                }

                #[test]
                fn backup_failure_should_fail() {
                    for (op, path) in [
                        (FsOp::Metadata, "/backup"),
                        (FsOp::Canonicalize, "/backup"),
                        (FsOp::Canonicalize, NAME),
                        (FsOp::Metadata, "/.backup"),
                        (FsOp::Remove, "/.backup"),
                        (FsOp::Open, "/backup"),
                        (FsOp::Write, "/backup"),
                        (FsOp::Read, NAME),
                    ] {
                        let platform = platform();
                        platform.fs().write("/backup", [9]).unwrap();
                        platform.fs().write("/.backup", [9]).unwrap();
                        let storage = written(&platform);
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        assert!(
                            storage.backup("/backup").is_err(),
                            "op: {op:?}, path: {path}"
                        );
                    }
                }
            }

            mod copy {
                use super::*;

                #[test]
                fn copy_should_create_independent_storage_named_name() {
                    crate::storage::tests::storage_data::copy::copy_should_create_independent_storage_named_name(
                        open(&platform()),
                        "/copy",
                    );
                }

                #[test]
                fn copy_should_not_replay_log_of_storage_previously_at_name() {
                    // GIVEN a storage killed with an unflushed write at the name
                    let platform = platform();
                    stale_storage(&platform, "/copy");
                    let storage = written(&platform);

                    // WHEN the storage is copied to the name
                    let copy = storage.copy("/copy").unwrap();

                    // THEN the copy holds the bytes of the storage
                    assert_eq!(read(&copy), vec![1, 2, 3]);
                }

                #[test]
                fn copy_failure_should_fail() {
                    let platform = platform();
                    let storage = written(&platform);
                    platform
                        .fs()
                        .set_io_error(FsOp::Open, "/copy", ErrorKind::Other);

                    assert!(storage.copy("/copy").is_err());
                }
            }

            mod flush {
                use super::*;

                #[test]
                fn flush_should_keep_written_bytes_on_recover() {
                    crate::storage::tests::storage_data::flush::flush_should_keep_written_bytes_on_recover(open(
                        &platform(),
                    ));
                }

                #[test]
                fn flush_should_make_writes_survive_crash_only_in_commit_mode() {
                    for (mode, expected) in
                        [(SyncMode::Commit, vec![1, 2, 3]), (SyncMode::None, vec![])]
                    {
                        let platform = platform();
                        let mut storage = open(&platform);
                        storage.set_sync_mode(mode);
                        storage.write(0, &[1, 2, 3]).unwrap();

                        storage.flush().unwrap();

                        platform.fs().crash();
                        assert_eq!(read(&open(&platform)), expected, "mode: {mode}");
                    }
                }

                #[test]
                fn flush_should_truncate_log_only_in_commit_mode() {
                    for (mode, truncated) in [(SyncMode::Commit, true), (SyncMode::None, false)] {
                        let platform = platform();
                        let mut storage = open(&platform);
                        storage.set_sync_mode(mode);
                        storage.write(0, &[1, 2, 3]).unwrap();

                        storage.flush().unwrap();

                        assert!(storage.wal.is_empty(), "mode: {mode}");
                        assert_eq!(
                            platform.fs().metadata(WAL).unwrap().len == 0,
                            truncated,
                            "mode: {mode}"
                        );
                    }
                }

                #[test]
                fn flush_without_writes_should_do_nothing() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.set_sync_mode(SyncMode::Commit);
                    platform
                        .fs()
                        .set_io_error(FsOp::SyncData, "", ErrorKind::Other);
                    platform
                        .fs()
                        .set_io_error(FsOp::Write, "", ErrorKind::Other);

                    assert_eq!(storage.flush(), Ok(()));
                }

                #[test]
                fn flush_failure_should_fail() {
                    for (mode, op, path) in [
                        (SyncMode::Commit, FsOp::SyncData, NAME),
                        (SyncMode::None, FsOp::Write, WAL),
                        (SyncMode::Commit, FsOp::SetLen, WAL),
                        (SyncMode::Commit, FsOp::SyncData, WAL),
                    ] {
                        let platform = platform();
                        let mut storage = open(&platform);
                        storage.set_sync_mode(mode);
                        storage.write(0, &[1]).unwrap();
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        assert!(
                            storage.flush().is_err(),
                            "mode: {mode}, op: {op:?}, path: {path}"
                        );
                    }
                }
            }

            mod is_empty {
                use super::*;

                #[test]
                fn is_empty_should_be_true_only_without_bytes() {
                    crate::storage::tests::storage_data::is_empty::is_empty_should_be_true_only_without_bytes(open(
                        &platform(),
                    ));
                }
            }

            mod set_sync_mode {
                use super::*;

                #[test]
                fn set_sync_mode_should_set_mode_if_supported() {
                    crate::storage::tests::storage_data::set_sync_mode::set_sync_mode_should_set_mode_if_supported(
                        open(&platform()),
                        true,
                    );
                }
            }

            mod sync_mode {
                use super::*;

                #[test]
                fn sync_mode_should_default_to_none() {
                    crate::storage::tests::storage_data::sync_mode::sync_mode_should_default_to_none(open(&platform()));
                }
            }

            mod sync {
                use super::*;

                #[test]
                fn sync_should_succeed() {
                    crate::storage::tests::storage_data::sync::sync_should_succeed(open(
                        &platform(),
                    ));
                }

                #[test]
                fn sync_should_make_bytes_survive_crash() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2]).unwrap();

                    storage.sync().unwrap();

                    platform.fs().crash();
                    assert_eq!(read(&open(&platform)), vec![1, 2]);
                }

                #[test]
                fn sync_failure_should_fail() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    platform
                        .fs()
                        .set_io_error(FsOp::SyncData, NAME, ErrorKind::Other);

                    assert!(storage.sync().is_err());
                }
            }

            mod len {
                use super::*;

                #[test]
                fn len_should_cover_written_bytes() {
                    crate::storage::tests::storage_data::len::len_should_cover_written_bytes(open(
                        &platform(),
                    ));
                }
            }

            mod name {
                use super::*;

                #[test]
                fn name_should_return_name() {
                    crate::storage::tests::storage_data::name::name_should_return_name(
                        open(&platform()),
                        NAME,
                    );
                }
            }

            mod new {
                use super::*;

                #[test]
                fn new_should_create_empty_storage_named_name() {
                    crate::storage::tests::storage_data::new::new_should_create_empty_storage_named_name::<TestFileStorage>();
                }
            }

            mod read {
                use super::*;

                #[test]
                fn read_should_return_bytes_at_pos() {
                    crate::storage::tests::storage_data::read::read_should_return_bytes_at_pos(
                        open(&platform()),
                    );
                }

                #[test]
                fn read_out_of_bounds_should_fail() {
                    crate::storage::tests::storage_data::read::read_out_of_bounds_should_fail(
                        open(&platform()),
                    );
                }
            }

            mod rename {
                use super::*;

                #[test]
                fn rename_should_change_name_and_keep_bytes() {
                    crate::storage::tests::storage_data::rename::rename_should_change_name_and_keep_bytes(
                        open(&platform()),
                        "/renamed",
                    );
                }

                #[test]
                fn rename_should_move_files_with_log() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2]).unwrap();
                    storage.flush().unwrap();
                    storage.write(0, &[3]).unwrap();

                    storage.rename("/renamed").unwrap();

                    for (path, exists) in [
                        (NAME, false),
                        (WAL, false),
                        ("/renamed", true),
                        ("/.renamed", true),
                    ] {
                        assert_eq!(platform.fs().exists(path).unwrap(), exists, "path: {path}");
                    }
                    storage.recover_from_wal().unwrap();
                    assert_eq!(read(&storage), vec![1, 2]);
                }

                #[test]
                fn rename_failure_should_keep_name_and_files() {
                    for path in [NAME, WAL] {
                        let platform = platform();
                        let mut storage = open(&platform);
                        platform
                            .fs()
                            .set_io_error(FsOp::Rename, path, ErrorKind::Other);

                        assert!(storage.rename("/renamed").is_err(), "path: {path}");

                        assert_eq!(storage.name(), NAME, "path: {path}");
                        assert!(platform.fs().exists(NAME).unwrap(), "path: {path}");
                        assert!(!platform.fs().exists("/renamed").unwrap(), "path: {path}");
                    }
                }
            }

            mod resize {
                use super::*;

                #[test]
                fn resize_should_zero_new_bytes_and_cut_off_others() {
                    crate::storage::tests::storage_data::resize::resize_should_zero_new_bytes_and_cut_off_others(open(
                        &platform(),
                    ));
                }

                #[test]
                fn resize_failure_should_fail_without_resizing() {
                    for (op, path, len) in [
                        (FsOp::Read, NAME, 1),
                        (FsOp::Write, WAL, 1),
                        (FsOp::Write, WAL, 5),
                        (FsOp::SetLen, NAME, 1),
                    ] {
                        let platform = platform();
                        let mut storage = open(&platform);
                        storage.write(0, &[1, 2, 3]).unwrap();
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        assert!(
                            storage.resize(len).is_err(),
                            "op: {op:?}, path: {path}, len: {len}"
                        );

                        assert_eq!(storage.len(), 3, "op: {op:?}, path: {path}, len: {len}");
                    }
                }
            }

            mod write {
                use super::*;

                #[test]
                fn write_should_overwrite_and_extend() {
                    crate::storage::tests::storage_data::write::write_should_overwrite_and_extend(
                        open(&platform()),
                    );
                }

                #[test]
                fn write_past_end_should_zero_gap() {
                    crate::storage::tests::storage_data::write::write_past_end_should_zero_gap(
                        open(&platform()),
                    );
                }

                #[test]
                fn write_empty_should_do_nothing() {
                    crate::storage::tests::storage_data::write::write_empty_should_do_nothing(
                        open(&platform()),
                    );
                }

                #[test]
                fn write_failure_should_fail_without_writing() {
                    for (op, path, pos) in [
                        (FsOp::Read, NAME, 0),
                        (FsOp::Write, WAL, 0),
                        (FsOp::Write, WAL, 3),
                        (FsOp::Write, NAME, 0),
                    ] {
                        let platform = platform();
                        let mut storage = open(&platform);
                        storage.write(0, &[1, 2, 3]).unwrap();
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        assert!(
                            storage.write(pos, &[4, 5]).is_err(),
                            "op: {op:?}, path: {path}, pos: {pos}"
                        );

                        platform.fs().clear_faults();
                        assert_eq!(
                            read(&storage),
                            vec![1, 2, 3],
                            "op: {op:?}, path: {path}, pos: {pos}"
                        );
                    }
                }
            }

            mod recover_from_wal {
                use super::*;

                #[test]
                fn recover_from_wal_should_restore_flushed_bytes_if_logged() {
                    crate::storage::tests::storage_data::recover_from_wal::recover_from_wal_should_restore_flushed_bytes_if_logged(
                        open(&platform()),
                        true,
                    );
                }

                #[test]
                fn recover_from_wal_failure_should_fail() {
                    for (op, path) in [(FsOp::Read, WAL), (FsOp::Seek, NAME)] {
                        let platform = platform();
                        let mut storage = open(&platform);
                        storage.write(0, &[1]).unwrap();
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        assert!(
                            storage.recover_from_wal().is_err(),
                            "op: {op:?}, path: {path}"
                        );
                    }
                }
            }
        }

        mod drop {
            use super::*;

            #[test]
            fn drop_failure_should_keep_log_for_next_open() {
                let platform = platform();
                let mut storage = open(&platform);
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();
                storage.write(0, &[4]).unwrap();
                platform
                    .fs()
                    .set_io_error(FsOp::Write, NAME, ErrorKind::Other);

                drop(storage);

                platform.fs().clear_faults();
                assert_eq!(read(&open(&platform)), vec![1, 2, 3]);
            }
        }
    }
}
