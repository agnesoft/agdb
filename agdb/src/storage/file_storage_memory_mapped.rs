use super::StorageData;
use super::StorageSlice;
use super::file_storage::FileStorageImpl;
use super::memory_storage::MemoryStorageImpl;
use crate::DbError;
use agplatform::Platform;
use agplatform::PlatformImpl;
use std::sync::Arc;

/// The default implementation of the database storage implementing
/// [`StorageData`]. It combines the [`FileStorageImpl`] and [`MemoryStorageImpl`]
/// leveraging the former for the persistence and the latter for performance.
/// The read operations are implemented in terms of the [`MemoryStorageImpl`] only
/// and the write operations are implemented in terms of both [`FileStorageImpl`]
/// and [`MemoryStorageImpl`].
///
/// The file based storage is using write ahead logging (WAL) and single underlying
/// file. See [`FileStorageImpl`] for more details on the persistent storage.
/// The files are accessed through the platform `P`, see
/// [`FileStorageMemoryMapped`] for the production one.
pub struct FileStorageMemoryMappedImpl<P: Platform> {
    file: FileStorageImpl<P>,
    memory: MemoryStorageImpl<P>,
}

/// The [`FileStorageMemoryMappedImpl`] on the production [`PlatformImpl`].
pub type FileStorageMemoryMapped = FileStorageMemoryMappedImpl<PlatformImpl>;

impl<P: Platform> FileStorageMemoryMappedImpl<P> {
    /// Constructs or loads the storage `name` in the file system of the
    /// `platform`, see [`StorageData::new()`].
    pub fn with_platform(platform: Arc<P>, name: &str) -> Result<Self, DbError> {
        let file = FileStorageImpl::with_platform(platform.clone(), name)?;
        let buffer = file.read_all()?;

        Ok(Self {
            file,
            memory: MemoryStorageImpl::from_buffer_with_platform(platform, name, buffer),
        })
    }
}

impl<P: Platform + Default> StorageData for FileStorageMemoryMappedImpl<P> {
    fn backup(&self, name: &str) -> Result<(), DbError> {
        self.file.backup(name)
    }

    fn copy(&self, name: &str) -> Result<Self, DbError> {
        let file = self.file.copy(name)?;
        let buffer = if self.file.is_deferring() {
            file.read_all()?
        } else {
            self.memory.bytes().to_vec()
        };

        Ok(Self {
            file,
            memory: MemoryStorageImpl::from_buffer_with_platform(
                self.file.platform().clone(),
                name,
                buffer,
            ),
        })
    }

    fn flush(&mut self) -> Result<(), DbError> {
        self.file.flush_deferred(self.memory.bytes())
    }

    fn set_overlay_limit(&mut self, limit: u64) {
        self.file.set_overlay_limit(limit);
    }

    fn overlay_limit(&self) -> u64 {
        self.file.overlay_limit()
    }

    fn set_sync_mode(&mut self, mode: super::SyncMode) {
        self.file.set_sync_mode(mode);
    }

    fn sync_mode(&self) -> super::SyncMode {
        self.file.sync_mode()
    }

    fn sync(&mut self) -> Result<(), DbError> {
        self.file.sync()
    }

    fn len(&self) -> u64 {
        self.file.len()
    }

    fn name(&self) -> &str {
        self.file.name()
    }

    fn new(name: &str) -> Result<Self, DbError> {
        Self::with_platform(Arc::new(P::default()), name)
    }

    fn read(&'_ self, pos: u64, value_len: u64) -> Result<StorageSlice<'_>, DbError> {
        self.memory.read(pos, value_len)
    }

    fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
        self.file.rename(new_name)?;
        self.memory.rename(new_name)
    }

    fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
        self.file.defer_resize(new_len, self.memory.bytes());
        self.memory
            .resize(new_len)
            .and_then(|()| self.file.spill(self.memory.bytes()))
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        self.file
            .defer_write(pos, bytes.len() as u64, self.memory.bytes());
        self.memory
            .write(pos, bytes)
            .and_then(|()| self.file.spill(self.memory.bytes()))
    }

    fn recover_from_wal(&mut self) -> Result<(), DbError> {
        self.file.recover_from_wal()?;
        let buffer = self.file.read_all()?;
        self.memory = MemoryStorageImpl::from_buffer_with_platform(
            self.file.platform().clone(),
            self.file.name(),
            buffer,
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::Fs;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::io::ErrorKind;

    const NAME: &str = "/db";
    const WAL: &str = "/.db";

    type TestMappedStorage = FileStorageMemoryMappedImpl<TestPlatform>;

    fn platform() -> Arc<TestPlatform> {
        Arc::new(test_platform())
    }

    fn open(platform: &Arc<TestPlatform>) -> TestMappedStorage {
        FileStorageMemoryMappedImpl::with_platform(platform.clone(), NAME).unwrap()
    }

    mod file_storage_memory_mapped_impl {
        use super::*;

        mod with_platform {
            use super::*;

            #[test]
            fn with_platform_should_load_file_into_memory() {
                let platform = platform();
                platform.fs().write(NAME, [1, 2, 3]).unwrap();

                let storage = open(&platform);

                assert_eq!(storage.memory.bytes(), [1, 2, 3]);
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
                assert_eq!(storage.memory.bytes(), [1, 2, 3, 4]);
                drop(storage);
                assert_eq!(platform.fs().read(NAME).unwrap(), [1, 2, 3, 4]);
                assert!(platform.fs().read(WAL).unwrap().is_empty());
            }

            #[test]
            fn with_platform_failure_should_fail() {
                for op in [FsOp::Open, FsOp::Read] {
                    let platform = platform();
                    platform.fs().write(NAME, [1, 2, 3]).unwrap();
                    platform.fs().set_io_error(op, NAME, ErrorKind::Other);

                    assert!(
                        FileStorageMemoryMappedImpl::with_platform(platform.clone(), NAME).is_err(),
                        "op: {op:?}"
                    );
                }
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
                fn copy_with_deferred_writes_should_copy_flushed_bytes() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2]).unwrap();
                    storage.flush().unwrap();
                    storage.write(0, &[3]).unwrap();

                    let copy = storage.copy("/copy").unwrap();

                    assert_eq!(copy.memory.bytes(), [1, 2]);
                }

                #[test]
                fn copy_with_deferred_writes_failure_should_fail() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2]).unwrap();
                    storage.flush().unwrap();
                    storage.write(0, &[3]).unwrap();
                    platform
                        .fs()
                        .set_io_error(FsOp::Read, "/copy", ErrorKind::Other);

                    assert!(storage.copy("/copy").is_err());
                }

                #[test]
                fn copy_failure_should_fail() {
                    let platform = platform();
                    let storage = open(&platform);
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
                    crate::storage::tests::storage_data::flush::flush_should_keep_written_bytes_on_recover(open(&platform()));
                }

                #[test]
                fn flush_should_write_memory_to_file() {
                    // GIVEN a shrink followed by a growth and writes on both sides of it
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2, 3, 4, 5, 6]).unwrap();
                    storage.flush().unwrap();
                    storage.write(4, &[7]).unwrap();
                    storage.resize(2).unwrap();
                    storage.write(0, &[8]).unwrap();
                    storage.resize(4).unwrap();
                    storage.write(5, &[9]).unwrap();

                    // WHEN flushing
                    storage.flush().unwrap();

                    // THEN the file holds the bytes of the memory and the log nothing
                    assert_eq!(storage.memory.bytes(), [8, 2, 0, 0, 0, 9]);
                    assert_eq!(storage.file.read_all().unwrap(), [8, 2, 0, 0, 0, 9]);
                    drop(storage);
                    assert_eq!(open(&platform).memory.bytes(), [8, 2, 0, 0, 0, 9]);
                }

                #[test]
                fn flush_failure_should_keep_writes_deferred_to_next_flush() {
                    for (op, path) in [
                        (FsOp::Write, WAL),
                        (FsOp::SetLen, NAME),
                        (FsOp::Write, NAME),
                    ] {
                        let platform = platform();
                        let mut storage = open(&platform);
                        storage.write(0, &[1, 2, 3, 4]).unwrap();
                        storage.flush().unwrap();
                        storage.write(1, &[9]).unwrap();
                        storage.resize(2).unwrap();
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        assert!(storage.flush().is_err(), "op: {op:?}, path: {path}");

                        platform.fs().clear_faults();
                        storage.flush().unwrap();
                        assert_eq!(
                            storage.file.read_all().unwrap(),
                            [1, 9],
                            "op: {op:?}, path: {path}"
                        );
                    }
                }

                #[test]
                fn flush_failure_should_roll_back_on_reopen_after_crash() {
                    for (op, path) in [
                        (FsOp::Write, WAL),
                        (FsOp::SetLen, NAME),
                        (FsOp::Write, NAME),
                    ] {
                        let platform = platform();
                        let mut storage = open(&platform);
                        storage.write(0, &[1, 2, 3, 4]).unwrap();
                        storage.flush().unwrap();
                        storage.write(1, &[9]).unwrap();
                        storage.write(6, &[9]).unwrap();
                        storage.resize(2).unwrap();
                        platform.fs().set_io_error(op, path, ErrorKind::Other);
                        assert!(storage.flush().is_err(), "op: {op:?}, path: {path}");
                        platform.fs().clear_faults();

                        platform.fs().sync();
                        platform.fs().crash();

                        assert_eq!(
                            open(&platform).memory.bytes(),
                            [1, 2, 3, 4],
                            "op: {op:?}, path: {path}"
                        );
                    }
                }
            }

            mod is_empty {
                use super::*;

                #[test]
                fn is_empty_should_be_true_only_without_bytes() {
                    crate::storage::tests::storage_data::is_empty::is_empty_should_be_true_only_without_bytes(open(&platform()));
                }
            }

            mod set_overlay_limit {
                use super::*;

                #[test]
                fn set_overlay_limit_should_set_limit_if_supported() {
                    crate::storage::tests::storage_data::set_overlay_limit::set_overlay_limit_should_set_limit_if_supported(
                        open(&platform()),
                        true,
                    );
                }
            }

            mod overlay_limit {
                use super::*;

                #[test]
                fn overlay_limit_should_default_to_default_overlay_limit() {
                    crate::storage::tests::storage_data::overlay_limit::overlay_limit_should_default_to_default_overlay_limit(
                        open(&platform()),
                    );
                }
            }

            mod set_sync_mode {
                use super::*;

                #[test]
                fn set_sync_mode_should_set_mode_if_supported() {
                    crate::storage::tests::storage_data::set_sync_mode::set_sync_mode_should_set_mode_if_supported(open(&platform()), true);
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
                    crate::storage::tests::storage_data::new::new_should_create_empty_storage_named_name::<TestMappedStorage>();
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
                    crate::storage::tests::storage_data::rename::rename_should_change_name_and_keep_bytes(open(&platform()), "/renamed");
                }

                #[test]
                fn rename_failure_should_keep_name() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    platform
                        .fs()
                        .set_io_error(FsOp::Rename, NAME, ErrorKind::Other);

                    assert!(storage.rename("/renamed").is_err());

                    assert_eq!(storage.name(), NAME);
                    assert_eq!(storage.memory.name(), NAME);
                }
            }

            mod resize {
                use super::*;

                #[test]
                fn resize_should_zero_new_bytes_and_cut_off_others() {
                    crate::storage::tests::storage_data::resize::resize_should_zero_new_bytes_and_cut_off_others(open(&platform()));
                }

                #[test]
                fn resize_should_log_old_bytes_from_memory() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2, 3]).unwrap();
                    storage.flush().unwrap();
                    platform
                        .fs()
                        .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                    storage.resize(1).unwrap();

                    platform.fs().clear_faults();
                    storage.recover_from_wal().unwrap();
                    assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![1, 2, 3]);
                }

                #[test]
                fn resize_should_defer_resizing_file_to_flush() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2, 3]).unwrap();
                    storage.flush().unwrap();

                    storage.resize(1).unwrap();

                    assert_eq!(storage.file.read_all().unwrap(), [1, 2, 3]);
                    storage.flush().unwrap();
                    assert_eq!(storage.file.read_all().unwrap(), [1]);
                }

                #[test]
                fn resize_past_overlay_limit_should_write_file_keeping_log() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2, 3]).unwrap();
                    storage.flush().unwrap();
                    storage.set_overlay_limit(0);

                    storage.resize(5).unwrap();

                    assert_eq!(storage.file.read_all().unwrap(), [1, 2, 3, 0, 0]);
                    storage.recover_from_wal().unwrap();
                    assert_eq!(storage.memory.bytes(), [1, 2, 3]);
                }

                #[test]
                fn resize_failure_should_fail() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2, 3]).unwrap();
                    storage.flush().unwrap();
                    storage.set_overlay_limit(0);
                    platform
                        .fs()
                        .set_io_error(FsOp::SetLen, NAME, ErrorKind::Other);

                    assert!(storage.resize(1).is_err());

                    platform.fs().clear_faults();
                    storage.recover_from_wal().unwrap();
                    assert_eq!(storage.memory.bytes(), [1, 2, 3]);
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
                fn write_should_log_old_bytes_from_memory() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2, 3]).unwrap();
                    storage.flush().unwrap();
                    platform
                        .fs()
                        .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                    storage.write(1, &[4, 5, 6]).unwrap();

                    platform.fs().clear_faults();
                    storage.recover_from_wal().unwrap();
                    assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![1, 2, 3]);
                }

                #[test]
                fn write_should_defer_writing_file_to_flush() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2, 3]).unwrap();
                    storage.flush().unwrap();

                    storage.write(1, &[4, 5, 6]).unwrap();

                    assert_eq!(storage.file.read_all().unwrap(), [1, 2, 3]);
                    storage.flush().unwrap();
                    assert_eq!(storage.file.read_all().unwrap(), [1, 4, 5, 6]);
                }

                #[test]
                fn write_past_overlay_limit_should_write_file_keeping_log() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2, 3]).unwrap();
                    storage.flush().unwrap();
                    storage.set_overlay_limit(0);

                    storage.write(1, &[4, 5, 6]).unwrap();

                    assert_eq!(storage.file.read_all().unwrap(), [1, 4, 5, 6]);
                    storage.recover_from_wal().unwrap();
                    assert_eq!(storage.memory.bytes(), [1, 2, 3]);
                }

                #[test]
                fn write_failure_should_fail() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2, 3]).unwrap();
                    storage.flush().unwrap();
                    storage.set_overlay_limit(0);
                    platform
                        .fs()
                        .set_io_error(FsOp::Write, NAME, ErrorKind::Other);

                    assert!(storage.write(1, &[4, 5, 6]).is_err());

                    platform.fs().clear_faults();
                    storage.recover_from_wal().unwrap();
                    assert_eq!(storage.memory.bytes(), [1, 2, 3]);
                }
            }

            mod recover_from_wal {
                use super::*;

                #[test]
                fn recover_from_wal_should_restore_flushed_bytes_if_logged() {
                    crate::storage::tests::storage_data::recover_from_wal::recover_from_wal_should_restore_flushed_bytes_if_logged(open(&platform()), true);
                }

                #[test]
                fn recover_from_wal_should_restore_flushed_bytes_after_random_changes() {
                    let platform = platform();

                    crate::storage::tests::storage_data::recover_from_wal::recover_from_wal_should_restore_flushed_bytes_after_random_changes(
                        || open(&platform),
                        || {
                            platform.fs().clear_faults();
                            platform.fs().set_capacity(None);
                            platform.fs().sync();
                            platform.fs().crash();
                        },
                        |choice| {
                            let fs = platform.fs();
                            let used: u64 = [NAME, WAL]
                                .iter()
                                .map(|path| fs.metadata(path).map_or(0, |metadata| metadata.len))
                                .sum();

                            match choice % 3 {
                                0 => fs.set_io_error(FsOp::Write, NAME, ErrorKind::Other),
                                1 => fs.set_io_error(FsOp::Write, WAL, ErrorKind::Other),
                                _ => {
                                    fs.set_short_io(FsOp::Write, "", 8);
                                    fs.set_capacity(Some(used + choice / 3 % 64));
                                }
                            }
                        },
                    );
                }

                #[test]
                fn recover_from_wal_should_discard_deferred_writes() {
                    let platform = platform();
                    let mut storage = open(&platform);
                    storage.write(0, &[1, 2, 3]).unwrap();
                    storage.flush().unwrap();
                    storage.write(0, &[9]).unwrap();
                    storage.resize(5).unwrap();

                    storage.recover_from_wal().unwrap();

                    assert_eq!(storage.memory.bytes(), [1, 2, 3]);
                    storage.flush().unwrap();
                    assert_eq!(storage.file.read_all().unwrap(), [1, 2, 3]);
                }

                #[test]
                fn recover_from_wal_failure_should_fail() {
                    for (op, path) in [(FsOp::Read, WAL), (FsOp::Read, NAME)] {
                        // GIVEN a log written by a flush that failed to write the file
                        let platform = platform();
                        let mut storage = open(&platform);
                        storage.write(0, &[1]).unwrap();
                        storage.flush().unwrap();
                        storage.write(0, &[2]).unwrap();
                        platform
                            .fs()
                            .set_io_error(FsOp::Write, NAME, ErrorKind::Other);
                        assert!(storage.flush().is_err());
                        platform.fs().clear_faults();
                        platform.fs().set_io_error(op, path, ErrorKind::Other);

                        // WHEN recovering
                        // THEN reading the log or the file fails

                        assert!(
                            storage.recover_from_wal().is_err(),
                            "op: {op:?}, path: {path}"
                        );
                    }
                }
            }
        }
    }
}
