use super::storage_record::StorageRecord;
use super::storage_record_with_index::StorageRecordWithIndex;
use super::write_ahead_log_record::WriteAheadLogRecord;
use crate::db_error::DbError;

pub(crate) trait StorageData<T = Self> {
    fn begin_transaction(&mut self);
    fn clear_wal(&mut self) -> Result<(), DbError>;
    fn create_index(&mut self, position: u64, size: u64) -> i64;
    fn end_transaction(&mut self) -> bool;
    fn indexes_by_position(&self) -> Vec<i64>;
    fn insert_wal_record(&mut self, record: WriteAheadLogRecord) -> Result<(), DbError>;
    fn read_exact(&mut self, buffer: &mut Vec<u8>) -> Result<(), DbError>;
    fn record(&self, index: i64) -> Result<StorageRecord, DbError>;
    fn record_mut(&mut self, index: i64) -> &mut StorageRecord;
    fn remove_index(&mut self, index: i64);
    fn seek(&mut self, position: std::io::SeekFrom) -> Result<u64, DbError>;
    fn set_len(&mut self, len: u64) -> Result<(), DbError>;
    fn set_records(&mut self, records: Vec<StorageRecordWithIndex>);
    fn wal_records(&mut self) -> Result<Vec<WriteAheadLogRecord>, DbError>;
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), DbError>;
}
