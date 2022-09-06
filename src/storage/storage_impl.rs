use super::serialize::Serialize;
use super::storage_record::StorageRecord;
use super::storage_record_with_index::StorageRecordWithIndex;
use super::write_ahead_log_record::WriteAheadLogRecord;
use crate::db_error::DbError;

pub(crate) trait StorageImpl<T = Self> {
    fn append(&mut self, bytes: Vec<u8>) -> Result<(), DbError> {
        self.write(std::io::SeekFrom::End(0), bytes)
    }

    fn apply_wal(&mut self) -> Result<(), DbError> {
        let records = self.wal_records()?;

        if !records.is_empty() {
            for record in records.iter().rev() {
                self.apply_wal_record(record)?;
            }

            self.clear_wal()?;
        }

        Ok(())
    }

    fn apply_wal_record(&mut self, record: &WriteAheadLogRecord) -> Result<(), DbError> {
        if record.bytes.is_empty() {
            self.set_len(record.position)?;
        } else {
            self.seek(std::io::SeekFrom::Start(record.position))?;
            self.write_all(&record.bytes)?;
        }

        Ok(())
    }

    fn begin_transaction(&mut self);

    fn clear_wal(&mut self) -> Result<(), DbError>;

    fn copy_record(
        &mut self,
        index: i64,
        old_position: u64,
        size: u64,
        new_position: u64,
    ) -> Result<(), DbError> {
        let bytes = self.read(std::io::SeekFrom::Start(old_position), size)?;
        self.write(std::io::SeekFrom::Start(new_position), bytes)?;
        self.record_mut(index).position = new_position;

        Ok(())
    }
    fn copy_record_to_end(
        &mut self,
        from: u64,
        size: u64,
        record_index: i64,
        record_size: u64,
    ) -> Result<StorageRecord, DbError> {
        let new_position = self.seek(std::io::SeekFrom::End(0))?;
        let bytes = self.read(std::io::SeekFrom::Start(from), size)?;
        self.append(record_index.serialize())?;
        self.append(record_size.serialize())?;
        self.append(bytes)?;

        Ok(StorageRecord {
            position: new_position,
            size: record_size,
        })
    }

    fn create_index(&mut self, position: u64, size: u64) -> i64;

    fn end_transaction(&mut self) -> bool;

    fn ensure_record_size(
        &mut self,
        record: &mut StorageRecord,
        index: i64,
        offset: u64,
        value_size: usize,
    ) -> Result<(), DbError> {
        let new_size = offset + value_size as u64;

        if new_size > record.size {
            self.move_record_to_end(index, new_size, offset, record)?;
        }

        Ok(())
    }

    fn indexes_by_position(&self) -> Vec<i64>;
    fn insert_wal_record(&mut self, record: WriteAheadLogRecord) -> Result<(), DbError>;

    fn invalidate_record(&mut self, index: i64, position: u64) -> Result<(), DbError> {
        self.write(std::io::SeekFrom::Start(position), (-index).serialize())
    }

    fn move_record_to_end(
        &mut self,
        index: i64,
        new_size: u64,
        offset: u64,
        record: &mut StorageRecord,
    ) -> Result<(), DbError> {
        let old_position = record.position;
        *record = self.copy_record_to_end(
            record.position + std::mem::size_of::<StorageRecord>() as u64,
            core::cmp::min(record.size, offset),
            index,
            new_size,
        )?;
        self.invalidate_record(index, old_position)?;
        self.set_len(record.position + std::mem::size_of::<StorageRecord>() as u64 + new_size)?;
        *self.record_mut(index) = record.clone();

        Ok(())
    }

    fn read(&mut self, position: std::io::SeekFrom, size: u64) -> Result<Vec<u8>, DbError> {
        self.seek(position)?;
        let mut buffer = vec![0_u8; size as usize];
        self.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    fn read_exact(&mut self, buffer: &mut Vec<u8>) -> Result<(), DbError>;

    fn read_record(&mut self) -> Result<StorageRecordWithIndex, DbError> {
        const SIZE: u64 = std::mem::size_of::<i64>() as u64;
        const CURRENT: std::io::SeekFrom = std::io::SeekFrom::Current(0);

        let position = self.seek(CURRENT)?;
        let index = i64::deserialize(&self.read(CURRENT, SIZE)?)?;
        let size = u64::deserialize(&self.read(CURRENT, SIZE)?)?;

        self.seek(std::io::SeekFrom::Current(size as i64))?;

        Ok(StorageRecordWithIndex {
            index,
            position,
            size,
        })
    }

    fn read_records(&mut self) -> Result<(), DbError> {
        let mut records: Vec<StorageRecordWithIndex> = vec![];
        self.seek(std::io::SeekFrom::End(0))?;
        let size = self.seek(std::io::SeekFrom::Current(0))?;
        self.seek(std::io::SeekFrom::Start(0))?;

        while self.seek(std::io::SeekFrom::Current(0))? < size {
            records.push(self.read_record()?);
        }

        self.set_records(records);

        Ok(())
    }

    fn record(&self, index: i64) -> Result<StorageRecord, DbError>;
    fn record_mut(&mut self, index: i64) -> &mut StorageRecord;
    fn remove_index(&mut self, index: i64);
    fn seek(&mut self, position: std::io::SeekFrom) -> Result<u64, DbError>;
    fn set_len(&mut self, len: u64) -> Result<(), DbError>;
    fn set_records(&mut self, records: Vec<StorageRecordWithIndex>);

    fn shrink_index(&mut self, index: i64, current_pos: u64) -> Result<u64, DbError> {
        let record = self.record(index)?;
        let record_size = std::mem::size_of::<StorageRecord>() as u64 + record.size;

        if record.position != current_pos {
            self.copy_record(index, record.position, record_size, current_pos)?;
        } else {
            self.seek(std::io::SeekFrom::Current(record_size as i64))?;
        }

        self.seek(std::io::SeekFrom::Current(0))
    }

    fn shrink_indexes(&mut self, indexes: Vec<i64>) -> Result<u64, DbError> {
        let mut current_pos = self.seek(std::io::SeekFrom::Start(0))?;

        for index in indexes {
            current_pos = self.shrink_index(index, current_pos)?;
        }

        Ok(current_pos)
    }

    fn truncate(&mut self, size: u64) -> Result<(), DbError> {
        let current_size = self.seek(std::io::SeekFrom::End(0))?;

        if size < current_size {
            let bytes = self.read(std::io::SeekFrom::Start(size), current_size - size)?;
            self.insert_wal_record(WriteAheadLogRecord {
                position: size,
                bytes,
            })?;
            self.set_len(size)?;
        }

        Ok(())
    }

    fn validate_offset<V>(size: u64, offset: u64) -> Result<(), DbError> {
        if size < offset {
            return Err(DbError::Storage(
                "deserialization error: offset out of bounds".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_value_size<V>(size: u64, offset: u64) -> Result<(), DbError> {
        if size - offset < std::mem::size_of::<V>() as u64 {
            return Err(DbError::Storage(
                "deserialization error: value out of bounds".to_string(),
            ));
        }

        Ok(())
    }

    fn value_position(position: u64, offset: u64) -> std::io::SeekFrom {
        std::io::SeekFrom::Start(position + std::mem::size_of::<StorageRecord>() as u64 + offset)
    }

    fn value_read_size<V>(size: u64, offset: u64) -> Result<u64, DbError> {
        Self::validate_offset::<V>(size, offset)?;
        Self::validate_value_size::<V>(size, offset)?;

        Ok(std::mem::size_of::<V>() as u64)
    }

    fn wal_records(&mut self) -> Result<Vec<WriteAheadLogRecord>, DbError>;

    fn write(&mut self, position: std::io::SeekFrom, bytes: Vec<u8>) -> Result<(), DbError> {
        let current_end = self.seek(std::io::SeekFrom::End(0))?;
        let write_pos = self.seek(position)?;

        if write_pos < current_end {
            let orig_bytes = self.read(
                std::io::SeekFrom::Start(write_pos),
                std::cmp::min(bytes.len() as u64, current_end - write_pos),
            )?;
            self.insert_wal_record(WriteAheadLogRecord {
                position: write_pos,
                bytes: orig_bytes,
            })?;
        } else {
            self.insert_wal_record(WriteAheadLogRecord {
                position: current_end,
                bytes: vec![],
            })?;
        }

        self.seek(position)?;
        self.write_all(&bytes)
    }

    fn write_all(&mut self, bytes: &[u8]) -> Result<(), DbError>;
}
