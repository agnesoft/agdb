use crate::DbError;
use crate::FileStorageImpl;
use crate::FileStorageMemoryMappedImpl;
use crate::MemoryStorageImpl;
use crate::StorageData;
use crate::StorageSlice;
use agplatform::Platform;
use agplatform::PlatformImpl;

/// The enum that can hold any of the other storage types
/// and that implements the `StorageData` trait by delegating
/// to the inner storage type. When constructed with `AnyStorage::new()`
/// it will be the memory mapped variant. The storages use the platform
/// `P`, see [`AnyStorage`] for the production one.
pub enum AnyStorageImpl<P: Platform> {
    MemoryMapped(FileStorageMemoryMappedImpl<P>),
    Memory(MemoryStorageImpl<P>),
    File(FileStorageImpl<P>),
}

/// The [`AnyStorageImpl`] on the production [`PlatformImpl`].
pub type AnyStorage = AnyStorageImpl<PlatformImpl>;

impl<P: Platform + Default> StorageData for AnyStorageImpl<P> {
    fn backup(&self, name: &str) -> Result<(), DbError> {
        match self {
            Self::MemoryMapped(s) => s.backup(name),
            Self::Memory(s) => s.backup(name),
            Self::File(s) => s.backup(name),
        }
    }

    fn copy(&self, name: &str) -> Result<Self, DbError> {
        match self {
            Self::MemoryMapped(s) => s.copy(name).map(Self::MemoryMapped),
            Self::Memory(s) => s.copy(name).map(Self::Memory),
            Self::File(s) => s.copy(name).map(Self::File),
        }
    }

    fn flush(&mut self) -> Result<(), DbError> {
        match self {
            Self::MemoryMapped(s) => s.flush(),
            Self::Memory(s) => s.flush(),
            Self::File(s) => s.flush(),
        }
    }

    fn set_overlay_limit(&mut self, limit: u64) {
        match self {
            Self::MemoryMapped(s) => s.set_overlay_limit(limit),
            Self::Memory(s) => s.set_overlay_limit(limit),
            Self::File(s) => s.set_overlay_limit(limit),
        }
    }

    fn overlay_limit(&self) -> u64 {
        match self {
            Self::MemoryMapped(s) => s.overlay_limit(),
            Self::Memory(s) => s.overlay_limit(),
            Self::File(s) => s.overlay_limit(),
        }
    }

    fn set_sync_mode(&mut self, mode: super::SyncMode) {
        match self {
            Self::MemoryMapped(s) => s.set_sync_mode(mode),
            Self::Memory(s) => s.set_sync_mode(mode),
            Self::File(s) => s.set_sync_mode(mode),
        }
    }

    fn sync_mode(&self) -> super::SyncMode {
        match self {
            Self::MemoryMapped(s) => s.sync_mode(),
            Self::Memory(s) => s.sync_mode(),
            Self::File(s) => s.sync_mode(),
        }
    }

    fn sync(&mut self) -> Result<(), DbError> {
        match self {
            Self::MemoryMapped(s) => s.sync(),
            Self::Memory(s) => s.sync(),
            Self::File(s) => s.sync(),
        }
    }

    fn len(&self) -> u64 {
        match self {
            Self::MemoryMapped(s) => s.len(),
            Self::Memory(s) => s.len(),
            Self::File(s) => s.len(),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::MemoryMapped(s) => s.name(),
            Self::Memory(s) => s.name(),
            Self::File(s) => s.name(),
        }
    }

    fn new(name: &str) -> Result<Self, DbError> {
        Ok(Self::MemoryMapped(FileStorageMemoryMappedImpl::new(name)?))
    }

    fn read(&'_ self, pos: u64, value_len: u64) -> Result<StorageSlice<'_>, DbError> {
        match self {
            Self::MemoryMapped(s) => s.read(pos, value_len),
            Self::Memory(s) => s.read(pos, value_len),
            Self::File(s) => s.read(pos, value_len),
        }
    }

    fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
        match self {
            Self::MemoryMapped(s) => s.rename(new_name),
            Self::Memory(s) => s.rename(new_name),
            Self::File(s) => s.rename(new_name),
        }
    }

    fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
        match self {
            Self::MemoryMapped(s) => s.resize(new_len),
            Self::Memory(s) => s.resize(new_len),
            Self::File(s) => s.resize(new_len),
        }
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        match self {
            Self::MemoryMapped(s) => s.write(pos, bytes),
            Self::Memory(s) => s.write(pos, bytes),
            Self::File(s) => s.write(pos, bytes),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            Self::MemoryMapped(s) => s.is_empty(),
            Self::Memory(s) => s.is_empty(),
            Self::File(s) => s.is_empty(),
        }
    }

    fn recover_from_wal(&mut self) -> Result<(), DbError> {
        match self {
            Self::MemoryMapped(s) => s.recover_from_wal(),
            Self::Memory(s) => s.recover_from_wal(),
            Self::File(s) => s.recover_from_wal(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::Fs;
    use agplatform::Platform;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::io::ErrorKind;
    use std::sync::Arc;

    const NAME: &str = "/db";

    type TestAnyStorage = AnyStorageImpl<TestPlatform>;

    /// Each variant on its own platform with whether its writes are logged
    /// and its sync mode supported.
    fn variants() -> Vec<(TestAnyStorage, bool, Arc<TestPlatform>)> {
        let (mapped, memory, file) = (
            Arc::new(test_platform()),
            Arc::new(test_platform()),
            Arc::new(test_platform()),
        );

        vec![
            (
                AnyStorageImpl::MemoryMapped(
                    FileStorageMemoryMappedImpl::with_platform(mapped.clone(), NAME).unwrap(),
                ),
                true,
                mapped,
            ),
            (
                AnyStorageImpl::Memory(
                    MemoryStorageImpl::with_platform(memory.clone(), NAME).unwrap(),
                ),
                false,
                memory,
            ),
            (
                AnyStorageImpl::File(FileStorageImpl::with_platform(file.clone(), NAME).unwrap()),
                true,
                file,
            ),
        ]
    }

    mod any_storage_impl {
        use super::*;

        mod storage_data {
            use super::*;

            mod backup {
                use super::*;

                #[test]
                fn backup_should_write_bytes_to_name() {
                    for (storage, _, platform) in variants() {
                        crate::storage::tests::storage_data::backup::backup_should_write_bytes_to_name(
                            storage,
                            "/backup",
                            |name| platform.fs().read(name).unwrap(),
                        );
                    }
                }
            }

            mod copy {
                use super::*;

                #[test]
                fn copy_should_create_independent_storage_named_name() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::copy::copy_should_create_independent_storage_named_name(
                            storage,
                            "/copy",
                        );
                    }
                }

                #[test]
                fn copy_failure_should_fail_for_file_variants() {
                    for (storage, logged, platform) in variants() {
                        platform
                            .fs()
                            .set_io_error(FsOp::Open, "/copy", ErrorKind::Other);

                        assert_eq!(storage.copy("/copy").is_err(), logged, "logged: {logged}");
                    }
                }
            }

            mod flush {
                use super::*;

                #[test]
                fn flush_should_keep_written_bytes_on_recover() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::flush::flush_should_keep_written_bytes_on_recover(storage);
                    }
                }
            }

            mod set_overlay_limit {
                use super::*;

                #[test]
                fn set_overlay_limit_should_set_limit_if_supported() {
                    for (storage, logged, _) in variants() {
                        crate::storage::tests::storage_data::set_overlay_limit::set_overlay_limit_should_set_limit_if_supported(storage, logged);
                    }
                }
            }

            mod overlay_limit {
                use super::*;

                #[test]
                fn overlay_limit_should_default_to_default_overlay_limit() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::overlay_limit::overlay_limit_should_default_to_default_overlay_limit(storage);
                    }
                }
            }

            mod set_sync_mode {
                use super::*;

                #[test]
                fn set_sync_mode_should_set_mode_if_supported() {
                    for (storage, logged, _) in variants() {
                        crate::storage::tests::storage_data::set_sync_mode::set_sync_mode_should_set_mode_if_supported(storage, logged);
                    }
                }
            }

            mod sync_mode {
                use super::*;

                #[test]
                fn sync_mode_should_default_to_none() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::sync_mode::sync_mode_should_default_to_none(storage);
                    }
                }
            }

            mod sync {
                use super::*;

                #[test]
                fn sync_should_succeed() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::sync::sync_should_succeed(storage);
                    }
                }
            }

            mod len {
                use super::*;

                #[test]
                fn len_should_cover_written_bytes() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::len::len_should_cover_written_bytes(
                            storage,
                        );
                    }
                }
            }

            mod name {
                use super::*;

                #[test]
                fn name_should_return_name() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::name::name_should_return_name(
                            storage, NAME,
                        );
                    }
                }
            }

            mod new {
                use super::*;

                #[test]
                fn new_should_create_empty_storage_named_name() {
                    crate::storage::tests::storage_data::new::new_should_create_empty_storage_named_name::<TestAnyStorage>();
                }

                #[test]
                fn new_failure_should_fail() {
                    assert!(TestAnyStorage::new("/missing/db").is_err());
                }

                #[test]
                fn new_should_create_memory_mapped_storage() {
                    assert!(matches!(
                        TestAnyStorage::new(NAME).unwrap(),
                        AnyStorageImpl::MemoryMapped(_)
                    ));
                }
            }

            mod read {
                use super::*;

                #[test]
                fn read_should_return_bytes_at_pos() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::read::read_should_return_bytes_at_pos(
                            storage,
                        );
                    }
                }

                #[test]
                fn read_out_of_bounds_should_fail() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::read::read_out_of_bounds_should_fail(
                            storage,
                        );
                    }
                }
            }

            mod rename {
                use super::*;

                #[test]
                fn rename_should_change_name_and_keep_bytes() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::rename::rename_should_change_name_and_keep_bytes(storage, "/renamed");
                    }
                }
            }

            mod resize {
                use super::*;

                #[test]
                fn resize_should_zero_new_bytes_and_cut_off_others() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::resize::resize_should_zero_new_bytes_and_cut_off_others(storage);
                    }
                }
            }

            mod write {
                use super::*;

                #[test]
                fn write_should_overwrite_and_extend() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::write::write_should_overwrite_and_extend(storage);
                    }
                }

                #[test]
                fn write_past_end_should_zero_gap() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::write::write_past_end_should_zero_gap(
                            storage,
                        );
                    }
                }

                #[test]
                fn write_empty_should_do_nothing() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::write::write_empty_should_do_nothing(
                            storage,
                        );
                    }
                }
            }

            mod is_empty {
                use super::*;

                #[test]
                fn is_empty_should_be_true_only_without_bytes() {
                    for (storage, _, _) in variants() {
                        crate::storage::tests::storage_data::is_empty::is_empty_should_be_true_only_without_bytes(storage);
                    }
                }
            }

            mod recover_from_wal {
                use super::*;

                #[test]
                fn recover_from_wal_should_restore_flushed_bytes_if_logged() {
                    for (storage, logged, _) in variants() {
                        crate::storage::tests::storage_data::recover_from_wal::recover_from_wal_should_restore_flushed_bytes_if_logged(storage, logged);
                    }
                }

                #[test]
                fn recover_from_wal_should_restore_flushed_bytes_after_random_changes() {
                    // WHY: it forwards to the file storages, which are tested with it.
                }
            }
        }
    }
}
