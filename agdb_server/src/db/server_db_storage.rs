use agdb::DbError;
use agdb::FileStorage;
use agdb::FileStorageMemoryMapped;
use agdb::MemoryStorage;
use agdb::StorageData;
use agdb::StorageSlice;

pub(crate) enum ServerDbStorage {
    MemoryMapped(FileStorageMemoryMapped),
    Memory(MemoryStorage),
    File(FileStorage),
}

impl StorageData for ServerDbStorage {
    fn backup(&mut self, name: &str) -> Result<(), DbError> {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.backup(name),
            ServerDbStorage::Memory(s) => s.backup(name),
            ServerDbStorage::File(s) => s.backup(name),
        }
    }

    fn flush(&mut self) -> Result<(), DbError> {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.flush(),
            ServerDbStorage::Memory(s) => s.flush(),
            ServerDbStorage::File(s) => s.flush(),
        }
    }

    fn len(&self) -> u64 {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.len(),
            ServerDbStorage::Memory(s) => s.len(),
            ServerDbStorage::File(s) => s.len(),
        }
    }

    fn name(&self) -> &str {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.name(),
            ServerDbStorage::Memory(s) => s.name(),
            ServerDbStorage::File(s) => s.name(),
        }
    }

    fn new(name: &str) -> Result<Self, DbError> {
        let (db_type, name) = name.split_once(':').ok_or(DbError::from("Invalid server database name format, must be 'type:name'. Allowed types: mapped, memory, file."))?;

        match db_type {
            "mapped" => Ok(Self::MemoryMapped(FileStorageMemoryMapped::new(name)?)),
            "memory" => Ok(Self::Memory(MemoryStorage::new(name)?)),
            "file" => Ok(Self::File(FileStorage::new(name)?)),
            _ => Err(DbError::from(format!(
                "Invalid db type '{}', must be one of 'mapped', 'memory', 'file'.",
                db_type
            ))),
        }
    }

    fn read(&self, pos: u64, value_len: u64) -> Result<StorageSlice, DbError> {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.read(pos, value_len),
            ServerDbStorage::Memory(s) => s.read(pos, value_len),
            ServerDbStorage::File(s) => s.read(pos, value_len),
        }
    }

    fn resize(&mut self, new_len: u64) -> Result<(), DbError> {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.resize(new_len),
            ServerDbStorage::Memory(s) => s.resize(new_len),
            ServerDbStorage::File(s) => s.resize(new_len),
        }
    }

    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.write(pos, bytes),
            ServerDbStorage::Memory(s) => s.write(pos, bytes),
            ServerDbStorage::File(s) => s.write(pos, bytes),
        }
    }

    fn is_empty(&self) -> bool {
        match self {
            ServerDbStorage::MemoryMapped(s) => s.is_empty(),
            ServerDbStorage::Memory(s) => s.is_empty(),
            ServerDbStorage::File(s) => s.is_empty(),
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

    impl std::fmt::Debug for ServerDbStorage {
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
        let _test_file_dot = TestFile::new(".file_storage.agdb");
        let test_file2 = TestFile::new("file_storage_backup.agdb");
        let mut storage = ServerDbStorage::new(&format!("file:{}", test_file.0))?;
        format!("{:?}", storage);
        storage.backup(&test_file2.0)?;
        assert!(std::path::Path::new(&test_file2.0).exists());
        storage.flush()?;
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.name(), test_file.0);
        assert!(storage.read(0, 0)?.is_empty());
        storage.write(0, &[])?;
        storage.resize(0)?;
        Ok(())
    }

    #[test]
    fn mapped_storage() -> anyhow::Result<()> {
        let test_file = TestFile::new("mapped_storage.agdb");
        let _test_file_dot = TestFile::new(".mapped_storage.agdb");
        let test_file2 = TestFile::new("mapped_storage_backup.agdb");
        let mut storage = ServerDbStorage::new(&format!("mapped:{}", test_file.0))?;
        format!("{:?}", storage);
        storage.backup(&test_file2.0)?;
        assert!(std::path::Path::new(&test_file2.0).exists());
        storage.flush()?;
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.name(), test_file.0);
        assert!(storage.read(0, 0)?.is_empty());
        storage.write(0, &[])?;
        storage.resize(0)?;
        Ok(())
    }

    #[test]
    fn memory_storage() -> anyhow::Result<()> {
        let mut storage = ServerDbStorage::new("memory:db_test.agdb")?;
        format!("{:?}", storage);
        storage.backup("backup_test")?;
        storage.flush()?;
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert_eq!(storage.name(), "db_test.agdb");
        assert!(storage.read(0, 0)?.is_empty());
        storage.write(0, &[])?;
        storage.resize(0)?;
        Ok(())
    }

    #[test]
    fn invalid_db_name() {
        assert_eq!(
            ServerDbStorage::new("db.agdb").unwrap_err().description,
            "Invalid server database name format, must be 'type:name'. Allowed types: mapped, memory, file."
        );
    }

    #[test]
    fn invalid_db_type() {
        assert_eq!(
            ServerDbStorage::new("sometype:db.agdb")
                .unwrap_err()
                .description,
            "Invalid db type 'sometype', must be one of 'mapped', 'memory', 'file'."
        );
    }
}
