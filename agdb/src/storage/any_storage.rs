use crate::DbError;
use crate::FileStorage;
use crate::FileStorageMemoryMapped;
use crate::MemoryStorage;
use crate::StorageData;
use crate::StorageSlice;

/// The enum that can hold any of the other storage types
/// and that implements the `StorageData` trait by delegating
/// to the inner storage type. When constructed with `AnyStorage::new()`
/// it will be the memory mapped variant.
pub enum AnyStorage {
    MemoryMapped(FileStorageMemoryMapped),
    Memory(MemoryStorage),
    File(FileStorage),
}

impl StorageData for AnyStorage {
    fn backup(&self, name: &str) -> Result<(), DbError> {
        match self {
            AnyStorage::MemoryMapped(s) => s.backup(name),
            AnyStorage::Memory(s) => s.backup(name),
            AnyStorage::File(s) => s.backup(name),
        }
    }

    fn copy(&self, name: &str) -> Result<Self, DbError> {
        Ok(match self {
            AnyStorage::MemoryMapped(s) => AnyStorage::MemoryMapped(s.copy(name)?),
            AnyStorage::Memory(s) => AnyStorage::Memory(s.copy(name)?),
            AnyStorage::File(s) => AnyStorage::File(s.copy(name)?),
        })
    }

    fn flush(&mut self) -> Result<(), DbError> {
        match self {
            AnyStorage::MemoryMapped(s) => s.flush(),
            AnyStorage::Memory(s) => s.flush(),
            AnyStorage::File(s) => s.flush(),
        }
    }

    fn len(&self) -> u64 {
        match self {
            AnyStorage::MemoryMapped(s) => s.len(),
            AnyStorage::Memory(s) => s.len(),
            AnyStorage::File(s) => s.len(),
        }
    }

    fn name(&self) -> &str {
        match self {
            AnyStorage::MemoryMapped(s) => s.name(),
            AnyStorage::Memory(s) => s.name(),
            AnyStorage::File(s) => s.name(),
        }
    }

    fn new(name: &str) -> Result<Self, DbError> {
        Ok(Self::MemoryMapped(FileStorageMemoryMapped::new(name)?))
    }

    fn read(&'_ self, pos: u64, value_len: u64) -> Result<StorageSlice<'_>, DbError> {
        match self {
            AnyStorage::MemoryMapped(s) => s.read(pos, value_len),
            AnyStorage::Memory(s) => s.read(pos, value_len),
            AnyStorage::File(s) => s.read(pos, value_len),
        }
    }

    fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
        match self {
            AnyStorage::MemoryMapped(s) => s.rename(new_name),
            AnyStorage::Memory(s) => s.rename(new_name),
            AnyStorage::File(s) => s.rename(new_name),
        }
    }

    fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
        match self {
            AnyStorage::MemoryMapped(s) => s.resize(new_len),
            AnyStorage::Memory(s) => s.resize(new_len),
            AnyStorage::File(s) => s.resize(new_len),
        }
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        match self {
            AnyStorage::MemoryMapped(s) => s.write(pos, bytes),
            AnyStorage::Memory(s) => s.write(pos, bytes),
            AnyStorage::File(s) => s.write(pos, bytes),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            AnyStorage::MemoryMapped(s) => s.is_empty(),
            AnyStorage::Memory(s) => s.is_empty(),
            AnyStorage::File(s) => s.is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
