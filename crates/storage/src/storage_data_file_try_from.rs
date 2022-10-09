use crate::storage_data_file::StorageDataFile;
use agdb_db_error::DbError;
use agdb_storage_index::StorageRecords;
use agdb_write_ahead_log::WriteAheadLog;

impl TryFrom<String> for StorageDataFile {
    type Error = DbError;

    fn try_from(filename: String) -> Result<Self, Self::Error> {
        Ok(StorageDataFile {
            file: std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .read(true)
                .open(&filename)?,
            filename: filename.clone(),
            records: StorageRecords::default(),
            wal: WriteAheadLog::try_from(&filename)?,
            wal_filename: filename,
            transactions: 0,
        })
    }
}
