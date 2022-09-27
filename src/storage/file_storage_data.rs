use super::storage_data::StorageData;
use super::storage_record::StorageRecord;
use super::storage_record_with_index::StorageRecordWithIndex;
use super::storage_records::StorageRecords;
use super::write_ahead_log::WriteAheadLog;
use super::write_ahead_log_record::WriteAheadLogRecord;
use crate::DbError;

#[allow(dead_code)]
pub(crate) struct FileStorageData {
    pub(super) file: std::fs::File,
    pub(super) filename: String,
    pub(super) records: StorageRecords,
    pub(super) wal: WriteAheadLog,
    pub(super) wal_filename: String,
    pub(super) transactions: u64,
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
