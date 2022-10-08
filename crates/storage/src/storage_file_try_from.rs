use crate::storage_data_file::StorageDataFile;
use crate::storage_file::StorageFile;
use agdb_db_error::DbError;

impl TryFrom<String> for StorageFile {
    type Error = DbError;

    fn try_from(filename: String) -> Result<Self, Self::Error> {
        let mut storage = StorageFile {
            data: StorageDataFile::try_from(filename)?,
        };

        storage.apply_wal()?;
        storage.read_records()?;

        Ok(storage)
    }
}

impl TryFrom<&str> for StorageFile {
    type Error = DbError;

    fn try_from(filename: &str) -> Result<Self, Self::Error> {
        Self::try_from(filename.to_string())
    }
}
