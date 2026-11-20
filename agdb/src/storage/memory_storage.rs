use super::StorageData;
use super::StorageSlice;
use super::write_ahead_log::remove_wal;
use crate::DbError;
use crate::DbErrorType;
use agplatform::Fs;
use agplatform::Platform;
use agplatform::PlatformImpl;
use std::sync::Arc;

/// In-memory buffer implementing [`StorageData`].
///
/// Reads return a non-owning slice into the underlying buffer.
/// [`backup()`](StorageData::backup) writes the buffer to a file
/// through the platform `P`, see [`MemoryStorage`] for the production one,
/// while [`flush()`](StorageData::flush) is a no-op.
///
/// Offers unmatched performance but **no persistence**.
pub struct MemoryStorageImpl<P: Platform> {
    platform: Arc<P>,
    buffer: Vec<u8>,
    name: String,
}

/// The [`MemoryStorageImpl`] on the production [`PlatformImpl`].
pub type MemoryStorage = MemoryStorageImpl<PlatformImpl>;

impl MemoryStorage {
    pub fn from_buffer(name: &str, buffer: Vec<u8>) -> Self {
        Self::from_buffer_with_platform(Arc::new(PlatformImpl::default()), name, buffer)
    }
}

impl<P: Platform> MemoryStorageImpl<P> {
    /// Constructs the storage `name` with the `buffer` backed up to the
    /// file system of the `platform`.
    pub fn from_buffer_with_platform(platform: Arc<P>, name: &str, buffer: Vec<u8>) -> Self {
        Self {
            platform,
            buffer,
            name: name.to_string(),
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Constructs the storage `name` loading the file `name` from the file
    /// system of the `platform` if it exists, see [`StorageData::new()`].
    pub fn with_platform(platform: Arc<P>, name: &str) -> Result<Self, DbError> {
        let buffer = if let Ok(true) = platform.fs().exists(name) {
            platform.fs().read(name)?
        } else {
            vec![]
        };

        Ok(Self::from_buffer_with_platform(platform, name, buffer))
    }
}

impl<P: Platform + Default> StorageData for MemoryStorageImpl<P> {
    fn backup(&self, name: &str) -> Result<(), DbError> {
        remove_wal(&*self.platform, name)?;
        self.platform.fs().write(name, &self.buffer)?;
        Ok(())
    }

    fn copy(&self, name: &str) -> Result<Self, DbError> {
        Ok(Self::from_buffer_with_platform(
            self.platform.clone(),
            name,
            self.buffer.clone(),
        ))
    }

    fn len(&self) -> u64 {
        self.buffer.len() as u64
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn new(name: &str) -> Result<Self, DbError> {
        Self::with_platform(Arc::new(P::default()), name)
    }

    fn read(&'_ self, pos: u64, value_len: u64) -> Result<StorageSlice<'_>, DbError> {
        let end = pos.saturating_add(value_len);

        if end > self.len() {
            return Err(DbError::storage(
                DbErrorType::OutOfBounds,
                format!("Read ({pos}..{end}) out of bounds ({})", self.len()),
            ));
        }

        Ok(StorageSlice::from(&self.buffer[pos as usize..end as usize]))
    }

    fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
        self.name = new_name.to_string();
        Ok(())
    }

    fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
        self.buffer.resize(new_len as usize, 0);
        Ok(())
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        if bytes.is_empty() {
            return Ok(());
        }

        let current_len = self.len();
        let end = pos + bytes.len() as u64;

        if end <= current_len {
            self.buffer[pos as usize..end as usize].copy_from_slice(bytes);
        } else {
            self.buffer.resize(pos as usize, 0);
            self.buffer.extend_from_slice(bytes);
        }

        Ok(())
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

    type TestMemoryStorage = MemoryStorageImpl<TestPlatform>;

    fn platform() -> Arc<TestPlatform> {
        Arc::new(test_platform())
    }

    fn open(platform: &Arc<TestPlatform>) -> TestMemoryStorage {
        MemoryStorageImpl::with_platform(platform.clone(), NAME).unwrap()
    }

    mod memory_storage {
        use super::*;

        mod from_buffer {
            use super::*;

            #[test]
            fn from_buffer_should_use_buffer_and_name() {
                let storage = MemoryStorage::from_buffer(NAME, vec![1, 2]);

                assert_eq!(storage.name(), NAME);
                assert_eq!(storage.bytes(), [1, 2]);
            }
        }
    }

    mod memory_storage_impl {
        use super::*;

        mod from_buffer_with_platform {
            use super::*;

            #[test]
            fn from_buffer_with_platform_should_use_buffer_and_name() {
                let storage =
                    MemoryStorageImpl::from_buffer_with_platform(platform(), NAME, vec![1, 2]);

                assert_eq!(storage.name(), NAME);
                assert_eq!(storage.read(0, 2).unwrap().to_vec(), vec![1, 2]);
            }
        }

        mod bytes {
            use super::*;

            #[test]
            fn bytes_should_return_buffer() {
                let storage =
                    MemoryStorageImpl::from_buffer_with_platform(platform(), NAME, vec![1, 2]);

                assert_eq!(storage.bytes(), [1, 2]);
            }
        }

        mod with_platform {
            use super::*;

            #[test]
            fn with_platform_should_load_existing_file() {
                let platform = platform();
                platform.fs().write(NAME, [1, 2]).unwrap();

                let storage = open(&platform);

                assert_eq!(storage.bytes(), [1, 2]);
            }

            #[test]
            fn with_platform_unknown_file_should_be_empty() {
                let platform = platform();
                platform
                    .fs()
                    .set_io_error(FsOp::Metadata, NAME, ErrorKind::Other);

                assert!(open(&platform).is_empty());
            }

            #[test]
            fn with_platform_read_failure_should_fail() {
                let platform = platform();
                platform.fs().write(NAME, [1, 2]).unwrap();
                platform
                    .fs()
                    .set_io_error(FsOp::Read, NAME, ErrorKind::Other);

                assert!(MemoryStorageImpl::with_platform(platform.clone(), NAME).is_err());
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
                fn backup_should_remove_log_of_file_previously_at_name() {
                    let platform = platform();
                    platform.fs().write("/.backup", [9]).unwrap();

                    open(&platform).backup("/backup").unwrap();

                    assert!(!platform.fs().exists("/.backup").unwrap());
                }

                #[test]
                fn backup_failure_should_fail() {
                    let platform = platform();
                    platform.fs().write("/.backup", [9]).unwrap();
                    let storage = open(&platform);

                    assert!(storage.backup("/missing/backup").is_err());
                    platform
                        .fs()
                        .set_io_error(FsOp::Remove, "/.backup", ErrorKind::Other);
                    assert!(storage.backup("/backup").is_err());
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
            }

            mod flush {
                use super::*;

                #[test]
                fn flush_should_keep_written_bytes_on_recover() {
                    crate::storage::tests::storage_data::flush::flush_should_keep_written_bytes_on_recover(
                        open(&platform()),
                    );
                }
            }

            mod is_empty {
                use super::*;

                #[test]
                fn is_empty_should_be_true_only_without_bytes() {
                    crate::storage::tests::storage_data::is_empty::is_empty_should_be_true_only_without_bytes(
                        open(&platform()),
                    );
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
                    crate::storage::tests::storage_data::new::new_should_create_empty_storage_named_name::<TestMemoryStorage>();
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

                #[test]
                fn read_out_of_bounds_should_report_range() {
                    let storage =
                        MemoryStorageImpl::from_buffer_with_platform(platform(), NAME, vec![1, 2]);

                    assert_eq!(
                        storage.read(1, 2),
                        Err(DbError::storage(
                            DbErrorType::OutOfBounds,
                            "Read (1..3) out of bounds (2)"
                        ))
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
            }

            mod resize {
                use super::*;

                #[test]
                fn resize_should_zero_new_bytes_and_cut_off_others() {
                    crate::storage::tests::storage_data::resize::resize_should_zero_new_bytes_and_cut_off_others(
                        open(&platform()),
                    );
                }
            }

            mod set_sync_mode {
                use super::*;

                #[test]
                fn set_sync_mode_should_set_mode_if_supported() {
                    crate::storage::tests::storage_data::set_sync_mode::set_sync_mode_should_set_mode_if_supported(
                        open(&platform()),
                        false,
                    );
                }
            }

            mod sync_mode {
                use super::*;

                #[test]
                fn sync_mode_should_default_to_none() {
                    crate::storage::tests::storage_data::sync_mode::sync_mode_should_default_to_none(
                        open(&platform()),
                    );
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
            }

            mod recover_from_wal {
                use super::*;

                #[test]
                fn recover_from_wal_should_restore_flushed_bytes_if_logged() {
                    crate::storage::tests::storage_data::recover_from_wal::recover_from_wal_should_restore_flushed_bytes_if_logged(
                        open(&platform()),
                        false,
                    );
                }
            }
        }
    }
}
