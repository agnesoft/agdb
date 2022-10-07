use crate::storage_data_file::FileStorageData;
use crate::storage_impl::StorageImpl;
use crate::write_ahead_log::wal_filename;
use db_error::DbError;

pub type FileStorage = StorageImpl<FileStorageData>;

impl TryFrom<String> for FileStorage {
    type Error = DbError;

    fn try_from(filename: String) -> Result<Self, Self::Error> {
        let wal_filename = wal_filename(&filename);

        let mut storage = FileStorage {
            data: FileStorageData::try_from((filename, wal_filename))?,
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
