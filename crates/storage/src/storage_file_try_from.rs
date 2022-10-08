use crate::storage_data_file::FileStorageData;
use crate::storage_file::FileStorage;
use agdb_db_error::DbError;

impl TryFrom<String> for FileStorage {
    type Error = DbError;

    fn try_from(filename: String) -> Result<Self, Self::Error> {
        let mut storage = FileStorage {
            data: FileStorageData::try_from(filename)?,
        };

        storage.apply_wal()?;
        storage.read_records()?;

        Ok(storage)
    }
}

impl TryFrom<&str> for FileStorage {
    type Error = DbError;

    fn try_from(filename: &str) -> Result<Self, Self::Error> {
        Self::try_from(filename.to_string())
    }
}
