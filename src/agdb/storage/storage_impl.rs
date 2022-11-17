use crate::db::db_error::DbError;
use crate::storage::storage_data::StorageData;
use crate::storage::storage_index::StorageIndex;
use crate::storage::storage_record::StorageRecord;
use crate::storage::write_ahead_log::WriteAheadLogRecord;
use crate::storage::Storage;
use crate::utilities::serialize::Serialize;
use std::cmp::max;
use std::cmp::min;
use std::io::SeekFrom;

pub struct StorageImpl<Data: StorageData> {
    pub(crate) data: Data,
}

impl<Data: StorageData> StorageImpl<Data> {
    pub(crate) fn append(&mut self, bytes: &[u8]) -> Result<(), DbError> {
        self.write(SeekFrom::End(0), bytes)
    }

    pub(crate) fn apply_wal(&mut self) -> Result<(), DbError> {
        let records = self.data.wal_records()?;

        if !records.is_empty() {
            for record in records.iter().rev() {
                self.apply_wal_record(record)?;
            }

            self.data.clear_wal()?;
        }

        Ok(())
    }

    fn apply_wal_record(&mut self, record: &WriteAheadLogRecord) -> Result<(), DbError> {
        if record.bytes.is_empty() {
            self.data.set_len(record.position)?;
        } else {
            self.data.seek(SeekFrom::Start(record.position))?;
            self.data.write_all(&record.bytes)?;
        }

        Ok(())
    }

    fn copy_record(
        &mut self,
        index: &StorageIndex,
        old_position: u64,
        size: u64,
        new_position: u64,
    ) -> Result<(), DbError> {
        let bytes = self.read(SeekFrom::Start(old_position), size)?;
        self.write(SeekFrom::Start(new_position), &bytes)?;
        self.data.record_mut(index).position = new_position;

        Ok(())
    }
    fn copy_record_to_end(
        &mut self,
        from: u64,
        size: u64,
        record_index: &StorageIndex,
        record_size: u64,
    ) -> Result<StorageRecord, DbError> {
        let new_position = self.data.seek(SeekFrom::End(0))?;
        let bytes = self.read(SeekFrom::Start(from), size)?;

        let record = StorageRecord {
            index: record_index.clone(),
            position: new_position,
            size: record_size,
        };

        self.append(&record.serialize())?;
        self.append(&bytes)?;

        Ok(record)
    }

    pub(crate) fn ensure_record_size(
        &mut self,
        record: &mut StorageRecord,
        index: &StorageIndex,
        offset: u64,
        value_size: u64,
    ) -> Result<(), DbError> {
        let new_size = offset + value_size;

        if new_size > record.size {
            self.resize_record(index, new_size, offset, record)?;
        }

        Ok(())
    }

    fn erase_bytes(&mut self, position: u64, size: u64) -> Result<(), DbError> {
        self.write(SeekFrom::Start(position), &vec![0_u8; size as usize])
    }

    pub(crate) fn invalidate_record(&mut self, position: u64) -> Result<(), DbError> {
        self.write(
            SeekFrom::Start(position),
            &StorageIndex::from(-1_i64).serialize(),
        )
    }

    fn is_at_end(&mut self, record: &StorageRecord) -> Result<bool, DbError> {
        let file_size = self.data.seek(SeekFrom::End(0))?;

        Ok((record.position + StorageRecord::fixed_size() + record.size) == file_size)
    }

    fn move_record_to_end(
        &mut self,
        index: &StorageIndex,
        new_size: u64,
        offset: u64,
        record: &mut StorageRecord,
    ) -> Result<(), DbError> {
        let old_position = record.position;
        *record = self.copy_record_to_end(
            record.position + StorageRecord::fixed_size(),
            core::cmp::min(record.size, offset),
            index,
            new_size,
        )?;
        self.invalidate_record(old_position)?;

        Ok(())
    }

    pub(crate) fn move_bytes(&mut self, from: u64, to: u64, size: u64) -> Result<(), DbError> {
        let bytes = self.read(SeekFrom::Start(from), size)?;
        self.write(SeekFrom::Start(to), &bytes)?;

        if from < to {
            self.erase_bytes(from, min(size, to - from))?;
        } else {
            let position = max(to + size, from);
            self.erase_bytes(position, from + size - position)?;
        }

        Ok(())
    }

    pub(crate) fn read(&mut self, position: SeekFrom, size: u64) -> Result<Vec<u8>, DbError> {
        self.data.seek(position)?;
        let mut buffer = vec![0_u8; size as usize];
        self.data.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    fn read_record(&mut self) -> Result<StorageRecord, DbError> {
        const CURRENT: SeekFrom = SeekFrom::Current(0);

        let position = self.data.seek(CURRENT)?;
        let mut record =
            StorageRecord::deserialize(&self.read(CURRENT, StorageRecord::fixed_size())?)?;
        record.position = position;

        self.data.seek(SeekFrom::Current(record.size as i64))?;

        Ok(record)
    }

    pub(crate) fn read_records(&mut self) -> Result<(), DbError> {
        let mut records: Vec<StorageRecord> = vec![];
        self.data.seek(SeekFrom::End(0))?;
        let size = self.data.seek(SeekFrom::Current(0))?;
        self.data.seek(SeekFrom::Start(0))?;

        while self.data.seek(SeekFrom::Current(0))? < size {
            records.push(self.read_record()?);
        }

        self.data.set_records(records);

        Ok(())
    }

    pub(crate) fn resize_record(
        &mut self,
        index: &StorageIndex,
        new_size: u64,
        offset: u64,
        record: &mut StorageRecord,
    ) -> Result<(), DbError> {
        if self.is_at_end(record)? {
            record.size = new_size;
        } else {
            self.move_record_to_end(index, new_size, offset, record)?;
        }

        self.data
            .set_len(record.position + StorageRecord::fixed_size() + new_size)?;
        *self.data.record_mut(index) = record.clone();

        Ok(())
    }

    fn shrink_index(&mut self, index: &StorageIndex, current_pos: u64) -> Result<u64, DbError> {
        let record = self.data.record(index)?;
        let record_size = StorageRecord::fixed_size() + record.size;

        if record.position != current_pos {
            self.copy_record(index, record.position, record_size, current_pos)?;
        } else {
            self.data.seek(SeekFrom::Current(record_size as i64))?;
        }

        self.data.seek(SeekFrom::Current(0))
    }

    pub(crate) fn shrink_indexes(&mut self, indexes: Vec<StorageIndex>) -> Result<u64, DbError> {
        let mut current_pos = self.data.seek(SeekFrom::Start(0))?;

        for index in indexes {
            current_pos = self.shrink_index(&index, current_pos)?;
        }

        Ok(current_pos)
    }

    pub(crate) fn truncate(&mut self, size: u64) -> Result<(), DbError> {
        let current_size = self.data.seek(SeekFrom::End(0))?;

        if size < current_size {
            let bytes = self.read(SeekFrom::Start(size), current_size - size)?;
            self.data.insert_wal_record(WriteAheadLogRecord {
                position: size,
                bytes,
            })?;
            self.data.set_len(size)?;
        }

        Ok(())
    }

    pub(crate) fn validate_move_size(
        offset: u64,
        size: u64,
        record_size: u64,
    ) -> Result<(), DbError> {
        if record_size < (offset + size) {
            return Err(DbError::from("move size out of bounds"));
        }

        Ok(())
    }

    fn validate_offset(size: u64, offset: u64) -> Result<(), DbError> {
        if size < offset {
            return Err(DbError::from("deserialization error: offset out of bounds"));
        }

        Ok(())
    }

    fn validate_value_size(read_size: u64, max_size: u64) -> Result<(), DbError> {
        if max_size < read_size {
            return Err(DbError::from("deserialization error: value out of bounds"));
        }

        Ok(())
    }

    pub(crate) fn value_position(position: u64, offset: u64) -> SeekFrom {
        SeekFrom::Start(Self::value_position_u64(position, offset))
    }

    pub(crate) fn value_position_u64(position: u64, offset: u64) -> u64 {
        position + StorageRecord::fixed_size() + offset
    }

    pub(crate) fn value_read_size<V: Serialize>(size: u64, offset: u64) -> Result<u64, DbError> {
        Self::validate_offset(size, offset)?;

        let mut read_size = V::fixed_size();
        let max_size = size - offset;

        if read_size == 0 {
            read_size = max_size;
        }

        Self::validate_value_size(read_size, max_size)?;
        Ok(read_size)
    }

    pub(crate) fn write(&mut self, position: SeekFrom, bytes: &[u8]) -> Result<(), DbError> {
        let current_end = self.data.seek(SeekFrom::End(0))?;
        let write_pos = self.data.seek(position)?;

        if write_pos < current_end {
            let orig_bytes = self.read(
                SeekFrom::Start(write_pos),
                min(bytes.len() as u64, current_end - write_pos),
            )?;
            self.data.insert_wal_record(WriteAheadLogRecord {
                position: write_pos,
                bytes: orig_bytes,
            })?;
        } else {
            self.data.insert_wal_record(WriteAheadLogRecord {
                position: current_end,
                bytes: vec![],
            })?;
        }

        self.data.seek(position)?;
        self.data.write_all(bytes)
    }
}

impl<T: StorageData> Drop for StorageImpl<T> {
    fn drop(&mut self) {
        if self.apply_wal().is_ok() {
            let _ = self.data.clear_wal();
        }
    }
}

impl<Data: StorageData> Storage for StorageImpl<Data> {
    fn commit(&mut self) -> Result<(), DbError> {
        if self.data.end_transaction() {
            self.data.clear_wal()?;
        }

        Ok(())
    }

    fn insert<V: Serialize>(&mut self, value: &V) -> Result<StorageIndex, DbError> {
        self.transaction();
        let position = self.size()?;
        let bytes = value.serialize();
        let record = self.data.create_record(position, bytes.len() as u64);

        self.append(&record.serialize())?;
        self.append(&bytes)?;
        self.commit()?;

        Ok(record.index)
    }

    fn insert_at<V: Serialize>(
        &mut self,
        index: &StorageIndex,
        offset: u64,
        value: &V,
    ) -> Result<(), DbError> {
        self.transaction();
        let mut record = self.data.record(index)?;
        let bytes = V::serialize(value);
        self.ensure_record_size(&mut record, index, offset, bytes.len() as u64)?;
        self.write(Self::value_position(record.position, offset), &bytes)?;
        self.commit()
    }

    fn move_at(
        &mut self,
        index: &StorageIndex,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<(), DbError> {
        if offset_from == offset_to || size == 0 {
            return Ok(());
        }

        let mut record = self.data.record(index)?;
        Self::validate_move_size(offset_from, size, record.size)?;
        self.transaction();
        self.ensure_record_size(&mut record, index, offset_to, size)?;
        self.move_bytes(
            Self::value_position_u64(record.position, offset_from),
            Self::value_position_u64(record.position, offset_to),
            size,
        )?;
        self.commit()?;

        Ok(())
    }

    fn remove(&mut self, index: &StorageIndex) -> Result<(), DbError> {
        self.transaction();
        let position = self.data.record(index)?.position;
        self.invalidate_record(position)?;
        self.data.remove_index(index);
        self.commit()
    }

    fn resize_value(&mut self, index: &StorageIndex, new_size: u64) -> Result<(), DbError> {
        if new_size == 0 {
            return Err(DbError::from("value size cannot be 0"));
        }

        let mut record = self.data.record(index)?;

        if record.size != new_size {
            self.transaction();
            self.resize_record(index, new_size, new_size, &mut record)?;
            self.commit()?;
        }

        Ok(())
    }

    fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.transaction();
        let indexes = self.data.indexes_by_position();
        let size = self.shrink_indexes(indexes)?;
        self.truncate(size)?;
        self.commit()
    }

    fn size(&mut self) -> Result<u64, DbError> {
        self.data.seek(SeekFrom::End(0))
    }

    fn transaction(&mut self) {
        self.data.begin_transaction();
    }

    fn value<V: Serialize>(&mut self, index: &StorageIndex) -> Result<V, DbError> {
        let record = self.data.record(index)?;
        V::deserialize(&self.read(Self::value_position(record.position, 0), record.size)?)
    }

    fn value_at<V: Serialize>(&mut self, index: &StorageIndex, offset: u64) -> Result<V, DbError> {
        let record = self.data.record(index)?;
        let bytes = self.read(
            Self::value_position(record.position, offset),
            Self::value_read_size::<V>(record.size, offset)?,
        );

        V::deserialize(&bytes?)
    }

    fn value_size(&self, index: &StorageIndex) -> Result<u64, DbError> {
        Ok(self.data.record(index)?.size)
    }
}
