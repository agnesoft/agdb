use super::file_records::FileRecord;
use super::file_records::FileRecords;
use super::write_ahead_log::WriteAheadLog;
use super::write_ahead_log::WriteAheadLogRecord;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeFixedSized;
use crate::DbError;
use crate::DbIndex;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

pub struct FileStorageImpl {
    file: File,
    file_records: FileRecords,
    transactions: u64,
    wal: WriteAheadLog,
}

impl FileStorageImpl {
    pub fn begin_transaction(&mut self) {
        self.transactions += 1;
    }

    pub fn end_transaction(&mut self) -> Result<(), DbError> {
        if self.transactions != 0 {
            self.transactions -= 1;

            if self.transactions == 0 {
                self.wal.clear()?;
            }
        }

        Ok(())
    }

    pub fn records(&self) -> Vec<FileRecord> {
        self.file_records.records()
    }

    pub fn len(&mut self) -> Result<u64, DbError> {
        Ok(self.file.seek(SeekFrom::End(0))?)
    }

    pub fn new(filename: &String) -> Result<Self, DbError> {
        let mut data = FileStorageImpl {
            file: OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(filename)?,
            file_records: FileRecords::new(),
            transactions: 0,
            wal: WriteAheadLog::new(filename)?,
        };

        data.apply_wal()?;
        data.read_records()?;

        Ok(data)
    }

    pub fn new_record(&mut self, pos: u64, value_len: u64) -> FileRecord {
        self.file_records.new_record(pos, value_len)
    }

    pub fn read_exact(&mut self, pos: u64, value_len: u64) -> Result<Vec<u8>, DbError> {
        self.file.seek(SeekFrom::Start(pos))?;

        let mut buffer = vec![0_u8; value_len as usize];
        self.file.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    pub fn remove_index(&mut self, index: u64) {
        self.file_records.remove_index(index);
    }

    pub fn set_len(&mut self, len: u64) -> Result<(), DbError> {
        Ok(self.file.set_len(len)?)
    }

    pub fn set_pos(&mut self, index: u64, pos: u64) {
        self.file_records.set_pos(index, pos);
    }

    pub fn set_size(&mut self, index: u64, size: u64) {
        self.file_records.set_size(index, size);
    }

    pub fn truncate(&mut self, size: u64) -> Result<(), DbError> {
        let current_size = self.file.seek(SeekFrom::End(0))?;

        if size < current_size {
            self.record_wal(size, current_size - size)?;
            self.set_len(size)?;
        }

        Ok(())
    }

    pub fn record(&self, index: u64) -> Result<FileRecord, DbError> {
        self.file_records.record(index)
    }

    pub fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError> {
        self.record_wal(pos, bytes.len() as u64)?;
        self.file.seek(SeekFrom::Start(pos))?;

        Ok(self.file.write_all(bytes)?)
    }

    fn apply_wal_record(&mut self, record: WriteAheadLogRecord) -> Result<(), DbError> {
        if record.value.is_empty() {
            self.set_len(record.pos)
        } else {
            self.write(record.pos, &record.value)
        }
    }

    fn apply_wal(&mut self) -> Result<(), DbError> {
        for record in self.wal.records()? {
            self.apply_wal_record(record)?;
        }

        self.wal.clear()
    }

    fn record_wal(&mut self, pos: u64, size: u64) -> Result<(), DbError> {
        if pos == self.len()? {
            self.wal.insert(pos, vec![])
        } else {
            let bytes = self.read_exact(pos, size)?;

            self.wal.insert(pos, bytes)
        }
    }

    fn read_record(&mut self) -> Result<FileRecord, DbError> {
        let pos = self.file.seek(SeekFrom::Current(0))?;
        let bytes = self.read_exact(pos, DbIndex::fixed_serialized_size())?;
        let index = DbIndex::deserialize(&bytes)?;
        self.file.seek(SeekFrom::Start(
            pos + DbIndex::fixed_serialized_size() + index.meta(),
        ))?;

        Ok(FileRecord {
            index: index.value(),
            pos,
            size: index.meta(),
        })
    }

    fn read_records(&mut self) -> Result<(), DbError> {
        let mut records: Vec<FileRecord> = vec![FileRecord::default()];
        let len = self.len()?;
        self.file.seek(SeekFrom::Start(0))?;

        while self.file.seek(SeekFrom::Current(0))? < len {
            records.push(self.read_record()?);
        }

        self.file_records.set_records(records);

        Ok(())
    }
}

impl Drop for FileStorageImpl {
    fn drop(&mut self) {
        if self.apply_wal().is_ok() {
            let _ = self.wal.clear();
        }
    }
}
