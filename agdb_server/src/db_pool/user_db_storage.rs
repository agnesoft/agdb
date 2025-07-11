use agdb::DbError;
use agdb::FileStorage;
use agdb::FileStorageMemoryMapped;
use agdb::MemoryStorage;
use agdb::StorageData;
use agdb::StorageSlice;

pub(crate) enum UserDbStorage {
    MemoryMapped(FileStorageMemoryMapped),
    Memory(MemoryStorage),
    File(FileStorage),
}

impl StorageData for UserDbStorage {
    fn backup(&self, name: &str) -> Result<(), DbError> {
        match self {
            UserDbStorage::MemoryMapped(s) => s.backup(name),
            UserDbStorage::Memory(s) => s.backup(name),
            UserDbStorage::File(s) => s.backup(name),
        }
    }

    fn copy(&self, name: &str) -> Result<Self, DbError> {
        Ok(match self {
            UserDbStorage::MemoryMapped(s) => UserDbStorage::MemoryMapped(s.copy(name)?),
            UserDbStorage::Memory(s) => UserDbStorage::Memory(s.copy(name)?),
            UserDbStorage::File(s) => UserDbStorage::File(s.copy(name)?),
        })
    }

    fn flush(&mut self) -> Result<(), DbError> {
        match self {
            UserDbStorage::MemoryMapped(s) => s.flush(),
            UserDbStorage::Memory(s) => s.flush(),
            UserDbStorage::File(s) => s.flush(),
        }
    }

    fn len(&self) -> u64 {
        match self {
            UserDbStorage::MemoryMapped(s) => s.len(),
            UserDbStorage::Memory(s) => s.len(),
            UserDbStorage::File(s) => s.len(),
        }
    }

    fn name(&self) -> &str {
        match self {
            UserDbStorage::MemoryMapped(s) => s.name(),
            UserDbStorage::Memory(s) => s.name(),
            UserDbStorage::File(s) => s.name(),
        }
    }

    fn new(name: &str) -> Result<Self, DbError> {
        let (db_type, name) = name.split_once(':').ok_or(DbError::from("Invalid server database name format, must be 'type:name'. Allowed types: mapped, memory, file."))?;

        match db_type {
            "mapped" => Ok(Self::MemoryMapped(FileStorageMemoryMapped::new(name)?)),
            "memory" => Ok(Self::Memory(MemoryStorage::new(name)?)),
            "file" => Ok(Self::File(FileStorage::new(name)?)),
            _ => Err(DbError::from(format!(
                "Invalid db type '{db_type}', must be one of 'mapped', 'memory', 'file'."
            ))),
        }
    }

    fn read(&self, pos: u64, value_len: u64) -> Result<StorageSlice, DbError> {
        match self {
            UserDbStorage::MemoryMapped(s) => s.read(pos, value_len),
            UserDbStorage::Memory(s) => s.read(pos, value_len),
            UserDbStorage::File(s) => s.read(pos, value_len),
        }
    }

    fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
        match self {
            UserDbStorage::MemoryMapped(s) => s.rename(new_name),
            UserDbStorage::Memory(s) => s.rename(new_name),
            UserDbStorage::File(s) => s.rename(new_name),
        }
    }

    fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
        match self {
            UserDbStorage::MemoryMapped(s) => s.resize(new_len),
            UserDbStorage::Memory(s) => s.resize(new_len),
            UserDbStorage::File(s) => s.resize(new_len),
        }
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        match self {
            UserDbStorage::MemoryMapped(s) => s.write(pos, bytes),
            UserDbStorage::Memory(s) => s.write(pos, bytes),
            UserDbStorage::File(s) => s.write(pos, bytes),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            UserDbStorage::MemoryMapped(s) => s.is_empty(),
            UserDbStorage::Memory(s) => s.is_empty(),
            UserDbStorage::File(s) => s.is_empty(),
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

    impl std::fmt::Debug for UserDbStorage {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::MemoryMapped(_) => f.write_str("MemoryMapped"),
                Self::Memory(_) => f.write_str("Memory"),
                Self::File(_) => f.write_str("File"),
            }
        }
    }

    #[test]
    fn file_storage() -> anyhow::Result<()> {
        let test_file = TestFile::new("file_storage.agdb");
        let test_file_copy = TestFile::new("file_storage_rename_copy.agdb");
        let test_file_rename = TestFile::new("file_storage_rename.agdb");
        let _test_file_dot = TestFile::new(".file_storage.agdb");
        let test_file_copy_dot = TestFile::new(".file_storage_rename_copy.agdb");
        let _test_file_rename_dot = TestFile::new(".file_storage_rename.agdb");
        let test_file_backup = TestFile::new("file_storage_backup.agdb");
        let mut storage = UserDbStorage::new(&format!("file:{}", test_file.0))?;
        let _ = format!("{storage:?}");
        storage.backup(&test_file_backup.0)?;
        assert!(std::path::Path::new(&test_file_backup.0).exists());
        let other = storage.copy(&test_file_copy.0)?;
        assert_eq!(other.name(), test_file_copy.0);
        assert!(std::path::Path::new(&test_file_copy.0).exists());
        assert!(std::path::Path::new(&test_file_copy_dot.0).exists());
        storage.flush()?;
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.name(), test_file.0);
        assert!(storage.read(0, 0)?.is_empty());
        storage.rename(&test_file_rename.0)?;
        storage.write(0, &[])?;
        storage.resize(0)?;
        Ok(())
    }

    #[test]
    fn mapped_storage() -> anyhow::Result<()> {
        let test_file = TestFile::new("mapped_storage.agdb");
        let test_file_copy = TestFile::new("mapped_storage_copy.agdb");
        let test_file_rename = TestFile::new("mapped_storage_rename.agdb");
        let _test_file_dot = TestFile::new(".mapped_storage.agdb");
        let test_file_copy_dot = TestFile::new(".mapped_storage_copy.agdb");
        let _test_file_rename_dot = TestFile::new(".mapped_storage_rename.agdb");
        let test_file2 = TestFile::new("mapped_storage_backup.agdb");
        let mut storage = UserDbStorage::new(&format!("mapped:{}", test_file.0))?;
        let _ = format!("{storage:?}");
        storage.backup(&test_file2.0)?;
        assert!(std::path::Path::new(&test_file2.0).exists());
        let other = storage.copy(&test_file_copy.0)?;
        assert_eq!(other.name(), test_file_copy.0);
        assert!(std::path::Path::new(&test_file_copy.0).exists());
        assert!(std::path::Path::new(&test_file_copy_dot.0).exists());
        storage.flush()?;
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.name(), test_file.0);
        assert!(storage.read(0, 0)?.is_empty());
        storage.rename(&test_file_rename.0)?;
        storage.write(0, &[])?;
        storage.resize(0)?;
        Ok(())
    }

    #[test]
    fn memory_storage() -> anyhow::Result<()> {
        let test_file = TestFile::new("backup_test");
        let mut storage = UserDbStorage::new("memory:db_test.agdb")?;
        let _ = format!("{storage:?}");
        storage.backup(&test_file.0)?;
        let other = storage.copy("db_test_copy.agdb")?;
        assert_eq!(other.name(), "db_test_copy.agdb");
        storage.flush()?;
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.name(), "db_test.agdb");
        assert!(storage.read(0, 0)?.is_empty());
        storage.rename("new_name")?;
        storage.write(0, &[])?;
        storage.resize(0)?;
        Ok(())
    }

    #[test]
    fn invalid_db_name() {
        assert_eq!(
            UserDbStorage::new("db.agdb").unwrap_err().description,
            "Invalid server database name format, must be 'type:name'. Allowed types: mapped, memory, file."
        );
    }

    #[test]
    fn invalid_db_type() {
        assert_eq!(
            UserDbStorage::new("sometype:db.agdb")
                .unwrap_err()
                .description,
            "Invalid db type 'sometype', must be one of 'mapped', 'memory', 'file'."
        );
    }
}
