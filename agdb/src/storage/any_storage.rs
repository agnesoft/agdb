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
        Ok(match self {
            Self::MemoryMapped(s) => Self::MemoryMapped(s.copy(name)?),
            Self::Memory(s) => Self::Memory(s.copy(name)?),
            Self::File(s) => Self::File(s.copy(name)?),
        })
    }

    fn flush(&mut self) -> Result<(), DbError> {
        match self {
            Self::MemoryMapped(s) => s.flush(),
            Self::Memory(s) => s.flush(),
            Self::File(s) => s.flush(),
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
    use crate::FileStorage;
    use crate::MemoryStorage;

    struct TestFile(String);

    impl TestFile {
        fn new(name: &str) -> Self {
            let _ = std::fs::remove_file(name);
            Self(name.to_string())
        }
    }

    impl Drop for TestFile {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }

    impl std::fmt::Debug for AnyStorage {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::MemoryMapped(_) => f.write_str("MemoryMapped"),
                Self::Memory(_) => f.write_str("Memory"),
                Self::File(_) => f.write_str("File"),
            }
        }
    }

    #[test]
    fn file_storage() {
        let test_file = TestFile::new("file_storage.agdb");
        let test_file_copy = TestFile::new("file_storage_rename_copy.agdb");
        let test_file_rename = TestFile::new("file_storage_rename.agdb");
        let _test_file_dot = TestFile::new(".file_storage.agdb");
        let test_file_copy_dot = TestFile::new(".file_storage_rename_copy.agdb");
        let _test_file_rename_dot = TestFile::new(".file_storage_rename.agdb");
        let test_file_backup = TestFile::new("file_storage_backup.agdb");

        let mut storage = AnyStorage::File(FileStorage::new("file_storage.agdb").unwrap());
        let _ = format!("{storage:?}");
        storage.backup(&test_file_backup.0).unwrap();
        assert!(std::path::Path::new(&test_file_backup.0).exists());
        let other = storage.copy(&test_file_copy.0).unwrap();
        assert_eq!(other.name(), test_file_copy.0);
        assert!(std::path::Path::new(&test_file_copy.0).exists());
        assert!(std::path::Path::new(&test_file_copy_dot.0).exists());
        storage.flush().unwrap();
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.name(), test_file.0);
        assert!(storage.read(0, 0).unwrap().is_empty());
        storage.rename(&test_file_rename.0).unwrap();
        storage.write(0, &[]).unwrap();
        storage.resize(0).unwrap();
    }

    #[test]
    fn mapped_storage() {
        let test_file = TestFile::new("mapped_storage.agdb");
        let test_file_copy = TestFile::new("mapped_storage_copy.agdb");
        let test_file_rename = TestFile::new("mapped_storage_rename.agdb");
        let _test_file_dot = TestFile::new(".mapped_storage.agdb");
        let test_file_copy_dot = TestFile::new(".mapped_storage_copy.agdb");
        let _test_file_rename_dot = TestFile::new(".mapped_storage_rename.agdb");
        let test_file2 = TestFile::new("mapped_storage_backup.agdb");

        let mut storage = AnyStorage::new(&test_file.0).unwrap();
        let _ = format!("{storage:?}");
        storage.backup(&test_file2.0).unwrap();
        assert!(std::path::Path::new(&test_file2.0).exists());
        let other = storage.copy(&test_file_copy.0).unwrap();
        assert_eq!(other.name(), test_file_copy.0);
        assert!(std::path::Path::new(&test_file_copy.0).exists());
        assert!(std::path::Path::new(&test_file_copy_dot.0).exists());
        storage.flush().unwrap();
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.name(), test_file.0);
        assert!(storage.read(0, 0).unwrap().is_empty());
        storage.rename(&test_file_rename.0).unwrap();
        storage.write(0, &[]).unwrap();
        storage.resize(0).unwrap();
    }

    #[test]
    fn file_storage_recover_from_wal() {
        let test_file = TestFile::new("file_storage_recover_from_wal.agdb");
        let _test_file_dot = TestFile::new(".file_storage_recover_from_wal.agdb");

        let mut storage = AnyStorage::File(FileStorage::new(&test_file.0).unwrap());
        storage.write(0, &[1, 2, 3]).unwrap();
        storage.flush().unwrap();
        storage.write(0, &[4, 5, 6]).unwrap();
        storage.write(3, &[7]).unwrap();
        storage.recover_from_wal().unwrap();

        assert_eq!(storage.len(), 3);
        assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn mapped_storage_recover_from_wal() {
        let test_file = TestFile::new("mapped_storage_recover_from_wal.agdb");
        let _test_file_dot = TestFile::new(".mapped_storage_recover_from_wal.agdb");

        let mut storage = AnyStorage::new(&test_file.0).unwrap();
        storage.write(0, &[1, 2, 3]).unwrap();
        storage.flush().unwrap();
        storage.write(0, &[4, 5, 6]).unwrap();
        storage.write(3, &[7]).unwrap();
        storage.recover_from_wal().unwrap();

        assert_eq!(storage.len(), 3);
        assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn memory_storage_recover_from_wal() {
        let mut storage = AnyStorage::Memory(MemoryStorage::new("db_test.agdb").unwrap());
        storage.write(0, &[1, 2, 3]).unwrap();
        storage.recover_from_wal().unwrap();

        assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn memory_storage() {
        let test_file = TestFile::new("backup_test");

        let mut storage = AnyStorage::Memory(MemoryStorage::new("db_test.agdb").unwrap());
        let _ = format!("{storage:?}");
        storage.backup(&test_file.0).unwrap();
        let other = storage.copy("db_test_copy.agdb").unwrap();
        assert_eq!(other.name(), "db_test_copy.agdb");
        storage.flush().unwrap();
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.name(), "db_test.agdb");
        assert!(storage.read(0, 0).unwrap().is_empty());
        storage.rename("new_name").unwrap();
        storage.write(0, &[]).unwrap();
        storage.resize(0).unwrap();
    }
}
