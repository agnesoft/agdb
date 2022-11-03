use crate::db::db_error::DbError;
use crate::storage::storage_data::StorageData;
use crate::storage::storage_index::StorageIndex;
use crate::storage::storage_record::StorageRecord;
use crate::storage::storage_records::StorageRecords;
use crate::storage::write_ahead_log::WriteAheadLog;
use crate::storage::write_ahead_log::WriteAheadLogRecord;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

pub struct StorageDataFile {
    pub(crate) file: File,
    #[allow(dead_code)]
    pub(crate) filename: String,
    pub(crate) records: StorageRecords,
    pub(crate) wal: WriteAheadLog,
    #[allow(dead_code)]
    pub(crate) wal_filename: String,
    pub(crate) transactions: u64,
}

impl StorageData for StorageDataFile {
    fn begin_transaction(&mut self) {
        self.transactions += 1;
    }

    fn clear_wal(&mut self) -> Result<(), DbError> {
        self.wal.clear()
    }

    fn create_record(&mut self, position: u64, size: u64) -> StorageRecord {
        self.records.create(position, size)
    }

    fn end_transaction(&mut self) -> bool {
        if self.transactions != 0 {
            self.transactions -= 1;
        }

        self.transactions == 0
    }

    fn indexes_by_position(&self) -> Vec<StorageIndex> {
        self.records.indexes_by_position()
    }

    fn insert_wal_record(&mut self, record: WriteAheadLogRecord) -> Result<(), DbError> {
        self.wal.insert(record)
    }

    fn read_exact(&mut self, buffer: &mut Vec<u8>) -> Result<(), DbError> {
        Ok(Read::read_exact(&mut self.file, buffer)?)
    }

    fn record(&self, index: &StorageIndex) -> Result<StorageRecord, DbError> {
        Ok(self
            .records
            .get(index)
            .ok_or_else(|| DbError::from(format!("index '{}' not found", index.value())))?
            .clone())
    }

    fn record_mut(&mut self, index: &StorageIndex) -> &mut StorageRecord {
        self.records
            .get_mut(index)
            .expect("validated by previous call to FileStorage::record()")
    }

    fn remove_index(&mut self, index: &StorageIndex) {
        self.records.remove(index);
    }

    fn seek(&mut self, position: SeekFrom) -> Result<u64, DbError> {
        Ok(Seek::seek(&mut self.file, position)?)
    }

    fn set_len(&mut self, len: u64) -> Result<(), DbError> {
        Ok(self.file.set_len(len)?)
    }

    fn set_records(&mut self, records: Vec<StorageRecord>) {
        self.records = StorageRecords::from(records);
    }

    fn wal_records(&mut self) -> Result<Vec<WriteAheadLogRecord>, DbError> {
        self.wal.records()
    }

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), DbError> {
        Ok(Write::write_all(&mut self.file, bytes)?)
    }
}

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
