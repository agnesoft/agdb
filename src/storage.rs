mod file_storage;
mod file_storage_data;
mod hash_map_data;
mod hash_map_data_memory;
mod hash_map_data_storage;
mod hash_map_impl;
mod hash_map_iterator;
mod hash_map_key_value;
mod hash_map_meta_value;
mod hash_multi_map;
mod hash_multi_map_impl;
mod serialize;
mod stable_hash;
mod storage_data;
mod storage_hash_map;
mod storage_hash_multi_map;
mod storage_record;
mod storage_record_with_index;
mod storage_records;
mod storage_vec;
mod vec_iterator;
mod write_ahead_log;
mod write_ahead_log_record;

use crate::DbError;
use storage_record::StorageRecord;
use storage_record_with_index::StorageRecordWithIndex;
use write_ahead_log_record::WriteAheadLogRecord;

#[allow(unused_imports)]
pub(crate) use file_storage::FileStorage;
pub(crate) use file_storage_data::FileStorageData;
#[allow(unused_imports)]
pub(crate) use hash_multi_map::HashMultiMap;
pub(crate) use serialize::Serialize;
#[allow(unused_imports)]
pub(crate) use stable_hash::StableHash;
pub(crate) use storage_data::StorageData;
pub(crate) use storage_hash_map::StorageHashMap;
#[allow(unused_imports)]
pub(crate) use storage_hash_multi_map::StorageHashMultiMap;
#[allow(unused_imports)]
pub(crate) use storage_vec::StorageVec;

pub(crate) struct Storage<T: StorageData> {
    data: T,
}

#[allow(dead_code)]
impl<T: StorageData> Storage<T> {
    pub(crate) fn commit(&mut self) -> Result<(), DbError> {
        if self.data.end_transaction() {
            self.data.clear_wal()?;
        }

        Ok(())
    }

    pub(crate) fn insert<V: Serialize>(&mut self, value: &V) -> Result<i64, DbError> {
        self.transaction();
        let position = self.size()?;
        let bytes = value.serialize();
        let index = self.data.create_index(position, bytes.len() as u64);

        self.append(&index.serialize())?;
        self.append(&(bytes.len() as u64).serialize())?;
        self.append(&bytes)?;
        self.commit()?;

        Ok(index)
    }

    pub(crate) fn insert_at<V: Serialize>(
        &mut self,
        index: i64,
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

    pub(crate) fn move_at(
        &mut self,
        index: i64,
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

    pub(crate) fn remove(&mut self, index: i64) -> Result<(), DbError> {
        self.transaction();
        let position = self.data.record(index)?.position;
        self.invalidate_record(index, position)?;
        self.data.remove_index(index);
        self.commit()
    }

    pub(crate) fn resize_value(&mut self, index: i64, new_size: u64) -> Result<(), DbError> {
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

    pub(crate) fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.transaction();
        let indexes = self.data.indexes_by_position();
        let size = self.shrink_indexes(indexes)?;
        self.truncate(size)?;
        self.commit()
    }

    pub(crate) fn size(&mut self) -> Result<u64, DbError> {
        self.data.seek(std::io::SeekFrom::End(0))
    }

    pub(crate) fn transaction(&mut self) {
        self.data.begin_transaction();
    }

    pub(crate) fn value<V: Serialize>(&mut self, index: i64) -> Result<V, DbError> {
        let record = self.data.record(index)?;
        V::deserialize(&self.read(Self::value_position(record.position, 0), record.size)?)
    }

    pub(crate) fn value_at<V: Serialize>(&mut self, index: i64, offset: u64) -> Result<V, DbError> {
        let record = self.data.record(index)?;
        let bytes = self.read(
            Self::value_position(record.position, offset),
            Self::value_read_size::<V>(record.size, offset)?,
        );

        V::deserialize(&bytes?)
    }

    pub(crate) fn value_size(&self, index: i64) -> Result<u64, DbError> {
        Ok(self.data.record(index)?.size)
    }

    fn append(&mut self, bytes: &[u8]) -> Result<(), DbError> {
        self.write(std::io::SeekFrom::End(0), bytes)
    }

    fn apply_wal(&mut self) -> Result<(), DbError> {
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
            self.data.seek(std::io::SeekFrom::Start(record.position))?;
            self.data.write_all(&record.bytes)?;
        }

        Ok(())
    }

    fn copy_record(
        &mut self,
        index: i64,
        old_position: u64,
        size: u64,
        new_position: u64,
    ) -> Result<(), DbError> {
        let bytes = self.read(std::io::SeekFrom::Start(old_position), size)?;
        self.write(std::io::SeekFrom::Start(new_position), &bytes)?;
        self.data.record_mut(index).position = new_position;

        Ok(())
    }
    fn copy_record_to_end(
        &mut self,
        from: u64,
        size: u64,
        record_index: i64,
        record_size: u64,
    ) -> Result<StorageRecord, DbError> {
        let new_position = self.data.seek(std::io::SeekFrom::End(0))?;
        let bytes = self.read(std::io::SeekFrom::Start(from), size)?;
        self.append(&record_index.serialize())?;
        self.append(&record_size.serialize())?;
        self.append(&bytes)?;

        Ok(StorageRecord {
            position: new_position,
            size: record_size,
        })
    }

    fn ensure_record_size(
        &mut self,
        record: &mut StorageRecord,
        index: i64,
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
        self.write(
            std::io::SeekFrom::Start(position),
            &vec![0_u8; size as usize],
        )
    }

    fn invalidate_record(&mut self, index: i64, position: u64) -> Result<(), DbError> {
        self.write(std::io::SeekFrom::Start(position), &(-index).serialize())
    }

    fn is_at_end(&mut self, record: &StorageRecord) -> Result<bool, DbError> {
        let file_size = self.data.seek(std::io::SeekFrom::End(0))?;

        Ok((record.position + StorageRecord::serialized_size() + record.size) == file_size)
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
            record.position + StorageRecord::serialized_size(),
            core::cmp::min(record.size, offset),
            index,
            new_size,
        )?;
        self.invalidate_record(index, old_position)?;

        Ok(())
    }

    fn move_bytes(&mut self, from: u64, to: u64, size: u64) -> Result<(), DbError> {
        let bytes = self.read(std::io::SeekFrom::Start(from), size)?;
        self.write(std::io::SeekFrom::Start(to), &bytes)?;

        if from < to {
            self.erase_bytes(from, std::cmp::min(size, to - from))?;
        } else {
            let position = std::cmp::max(to + size, from);
            self.erase_bytes(position, from + size - position)?;
        }

        Ok(())
    }

    fn read(&mut self, position: std::io::SeekFrom, size: u64) -> Result<Vec<u8>, DbError> {
        self.data.seek(position)?;
        let mut buffer = vec![0_u8; size as usize];
        self.data.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    fn read_record(&mut self) -> Result<StorageRecordWithIndex, DbError> {
        let index_size: u64 = i64::serialized_size();
        const CURRENT: std::io::SeekFrom = std::io::SeekFrom::Current(0);

        let position = self.data.seek(CURRENT)?;
        let index = i64::deserialize(&self.read(CURRENT, index_size)?)?;
        let size = u64::deserialize(&self.read(CURRENT, index_size)?)?;

        self.data.seek(std::io::SeekFrom::Current(size as i64))?;

        Ok(StorageRecordWithIndex {
            index,
            position,
            size,
        })
    }

    fn read_records(&mut self) -> Result<(), DbError> {
        let mut records: Vec<StorageRecordWithIndex> = vec![];
        self.data.seek(std::io::SeekFrom::End(0))?;
        let size = self.data.seek(std::io::SeekFrom::Current(0))?;
        self.data.seek(std::io::SeekFrom::Start(0))?;

        while self.data.seek(std::io::SeekFrom::Current(0))? < size {
            records.push(self.read_record()?);
        }

        self.data.set_records(records);

        Ok(())
    }

    fn resize_record(
        &mut self,
        index: i64,
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
            .set_len(record.position + StorageRecord::serialized_size() + new_size)?;
        *self.data.record_mut(index) = record.clone();

        Ok(())
    }

    fn shrink_index(&mut self, index: i64, current_pos: u64) -> Result<u64, DbError> {
        let record = self.data.record(index)?;
        let record_size = StorageRecord::serialized_size() + record.size;

        if record.position != current_pos {
            self.copy_record(index, record.position, record_size, current_pos)?;
        } else {
            self.data
                .seek(std::io::SeekFrom::Current(record_size as i64))?;
        }

        self.data.seek(std::io::SeekFrom::Current(0))
    }

    fn shrink_indexes(&mut self, indexes: Vec<i64>) -> Result<u64, DbError> {
        let mut current_pos = self.data.seek(std::io::SeekFrom::Start(0))?;

        for index in indexes {
            current_pos = self.shrink_index(index, current_pos)?;
        }

        Ok(current_pos)
    }

    fn truncate(&mut self, size: u64) -> Result<(), DbError> {
        let current_size = self.data.seek(std::io::SeekFrom::End(0))?;

        if size < current_size {
            let bytes = self.read(std::io::SeekFrom::Start(size), current_size - size)?;
            self.data.insert_wal_record(WriteAheadLogRecord {
                position: size,
                bytes,
            })?;
            self.data.set_len(size)?;
        }

        Ok(())
    }

    fn validate_move_size(offset: u64, size: u64, record_size: u64) -> Result<(), DbError> {
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

    fn value_position(position: u64, offset: u64) -> std::io::SeekFrom {
        std::io::SeekFrom::Start(Self::value_position_u64(position, offset))
    }

    fn value_position_u64(position: u64, offset: u64) -> u64 {
        position + StorageRecord::serialized_size() + offset
    }

    fn value_read_size<V: Serialize>(size: u64, offset: u64) -> Result<u64, DbError> {
        Self::validate_offset(size, offset)?;

        let mut read_size = V::serialized_size();
        let max_size = size - offset;

        if read_size == 0 {
            read_size = max_size;
        }

        Self::validate_value_size(read_size, max_size)?;
        Ok(read_size)
    }

    fn write(&mut self, position: std::io::SeekFrom, bytes: &[u8]) -> Result<(), DbError> {
        let current_end = self.data.seek(std::io::SeekFrom::End(0))?;
        let write_pos = self.data.seek(position)?;

        if write_pos < current_end {
            let orig_bytes = self.read(
                std::io::SeekFrom::Start(write_pos),
                std::cmp::min(bytes.len() as u64, current_end - write_pos),
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

impl<T: StorageData> Drop for Storage<T> {
    fn drop(&mut self) {
        if self.apply_wal().is_ok() {
            let _ = self.data.clear_wal();
        }
    }
}
