use crate::storage_data::StorageData;
use crate::storage_record::StorageRecord;
use crate::storage_record_with_index::StorageRecordWithIndex;
use crate::storage_records::StorageRecords;
use crate::write_ahead_log::WriteAheadLog;
use crate::write_ahead_log_record::WriteAheadLogRecord;
use db_error::DbError;

#[allow(dead_code)]
pub struct FileStorageData {
    file: std::fs::File,
    filename: String,
    records: StorageRecords,
    wal: WriteAheadLog,
    wal_filename: String,
    transactions: u64,
}

impl StorageData for FileStorageData {
    fn begin_transaction(&mut self) {
        self.transactions += 1;
    }

    fn clear_wal(&mut self) -> Result<(), DbError> {
        self.wal.clear()
    }

    fn create_index(&mut self, position: u64, size: u64) -> i64 {
        self.records.create(position, size)
    }

    fn end_transaction(&mut self) -> bool {
        if self.transactions != 0 {
            self.transactions -= 1;
        }

        self.transactions == 0
    }

    fn indexes_by_position(&self) -> Vec<i64> {
        self.records.indexes_by_position()
    }

    fn insert_wal_record(&mut self, record: WriteAheadLogRecord) -> Result<(), DbError> {
        self.wal.insert(record)
    }

    fn read_exact(&mut self, buffer: &mut Vec<u8>) -> Result<(), DbError> {
        Ok(std::io::Read::read_exact(&mut self.file, buffer)?)
    }

    fn record(&self, index: i64) -> Result<StorageRecord, DbError> {
        Ok(self
            .records
            .get(index)
            .ok_or_else(|| DbError::from(format!("index '{}' not found", index)))?
            .clone())
    }

    fn record_mut(&mut self, index: i64) -> &mut StorageRecord {
        self.records
            .get_mut(index)
            .expect("validated by previous call to FileStorage::record()")
    }

    fn remove_index(&mut self, index: i64) {
        self.records.remove(index);
    }

    fn seek(&mut self, position: std::io::SeekFrom) -> Result<u64, DbError> {
        Ok(std::io::Seek::seek(&mut self.file, position)?)
    }

    fn set_len(&mut self, len: u64) -> Result<(), DbError> {
        Ok(self.file.set_len(len)?)
    }

    fn set_records(&mut self, records: Vec<StorageRecordWithIndex>) {
        self.records = StorageRecords::from(records);
    }

    fn wal_records(&mut self) -> Result<Vec<WriteAheadLogRecord>, DbError> {
        self.wal.records()
    }

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), DbError> {
        Ok(std::io::Write::write_all(&mut self.file, bytes)?)
    }
}

impl TryFrom<(String, String)> for FileStorageData {
    type Error = DbError;

    fn try_from(filenames: (String, String)) -> Result<Self, Self::Error> {
        Ok(FileStorageData {
            file: std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .read(true)
                .open(&filenames.0)?,
            filename: filenames.0,
            records: StorageRecords::default(),
            wal: WriteAheadLog::try_from(&filenames.1)?,
            wal_filename: filenames.1,
            transactions: 0,
        })
    }
}
