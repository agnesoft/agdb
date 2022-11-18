use crate::db::db_error::DbError;
use crate::old_storage::storage_index::StorageIndex;
use crate::old_storage::storage_record::StorageRecord;
use crate::old_storage::write_ahead_log::WriteAheadLogRecord;
use std::io::SeekFrom;

pub trait StorageData<T = Self> {
    fn begin_transaction(&mut self);
    fn clear_wal(&mut self) -> Result<(), DbError>;
    fn create_record(&mut self, position: u64, size: u64) -> StorageRecord;
    fn end_transaction(&mut self) -> bool;
    fn indexes_by_position(&self) -> Vec<StorageIndex>;
    fn insert_wal_record(&mut self, record: WriteAheadLogRecord) -> Result<(), DbError>;
    fn read_exact(&mut self, buffer: &mut Vec<u8>) -> Result<(), DbError>;
    fn record(&self, index: &StorageIndex) -> Result<StorageRecord, DbError>;
    fn record_mut(&mut self, index: &StorageIndex) -> &mut StorageRecord;
    fn remove_index(&mut self, index: &StorageIndex);
    fn seek(&mut self, position: SeekFrom) -> Result<u64, DbError>;
    fn set_len(&mut self, len: u64) -> Result<(), DbError>;
    fn set_records(&mut self, records: Vec<StorageRecord>);
    fn wal_records(&mut self) -> Result<Vec<WriteAheadLogRecord>, DbError>;
    fn write_all(&mut self, bytes: &[u8]) -> Result<(), DbError>;
}
