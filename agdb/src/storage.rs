pub mod file_storage;
pub mod file_storage_memory_mapped;
pub mod memory_storage;

mod storage_records;
mod write_ahead_log;

use self::storage_records::StorageRecord;
use self::storage_records::StorageRecords;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::DbError;
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct StorageIndex(pub u64);

impl From<u64> for StorageIndex {
    fn from(index: u64) -> Self {
        Self(index)
    }
}

impl Serialize for StorageIndex {
    fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self(u64::deserialize(bytes)?))
    }

    fn serialized_size(&self) -> u64 {
        self.0.serialized_size()
    }
}

impl SerializeStatic for StorageIndex {}

/// Convenience alias for `Cow<'a, [u8]>`.
pub type StorageSlice<'a> = Cow<'a, [u8]>;

/// Minimum set of data operations required by the database
/// to store & retrieve data.
pub trait StorageData: Sized {
    /// Copy the underlying data storage to a new `name`. The
    /// default implementation does nothing. File implementations
    /// might need to copy the underlying file(s).
    fn backup(&mut self, _name: &str) -> Result<(), DbError> {
        Ok(())
    }

    /// Flushes any buffers to the underlying storage (e.g. file). The
    /// default implementation does nothing.
    fn flush(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    /// Convenience method that returns `len() == 0`.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the length of the underlying storage in bytes.
    fn len(&self) -> u64;

    /// Returns the name this storage was constructed with.
    fn name(&self) -> &str;

    /// Constructs or loads the storage `name`. The `name` might be
    /// a file name or other identifier.
    fn new(name: &str) -> Result<Self, DbError>;

    /// Reads `value_len` bytes starting at `pos`. Returns [`StorageSlice`]
    /// (COW).
    fn read(&self, pos: u64, value_len: u64) -> Result<StorageSlice, DbError>;

    /// Changes the name of the storage changing also the names of the files
    /// (if the storage is file based).
    fn rename(&mut self, new_name: &str) -> Result<(), DbError>;

    /// Resizes the underlying storage to `new_len`. If the storage is enlarged as
    /// a result the new bytes should be initialized to `0_u8`.
    fn resize(&mut self, new_len: u64) -> Result<(), DbError>;

    /// Writes the `bytes` to the underlying storage at `pos`. The implementation
    /// must handle the case where the `pos + bytes.len()` exceeds the current
    /// [`len()`](#method.len).
    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError>;
}

pub struct Storage<D: StorageData> {
    data: D,
    records: StorageRecords,
    transactions: u64,
}

impl<D: StorageData> Storage<D> {
    pub fn new(name: &str) -> Result<Self, DbError> {
        let mut storage = Self {
            data: D::new(name)?,
            records: StorageRecords::new(),
            transactions: 0,
        };

        storage.read_records()?;

        Ok(storage)
    }

    pub fn backup(&mut self, name: &str) -> Result<(), DbError> {
        self.data.backup(name)
    }

    pub fn commit(&mut self, id: u64) -> Result<(), DbError> {
        self.end_transaction(id)
    }

    pub fn insert<T: Serialize>(&mut self, value: &T) -> Result<StorageIndex, DbError> {
        self.insert_bytes(&value.serialize())
    }

    pub fn insert_at<T: Serialize>(
        &mut self,
        index: StorageIndex,
        offset: u64,
        value: &T,
    ) -> Result<(), DbError> {
        self.insert_bytes_at(index, offset, &value.serialize())
    }

    pub fn insert_bytes(&mut self, bytes: &[u8]) -> Result<StorageIndex, DbError> {
        let len = self.len();
        let record = self.new_record(len, bytes.len() as u64);

        let id = self.transaction();
        self.write_record(&record)?;
        self.append(bytes)?;
        self.commit(id)?;

        Ok(StorageIndex::from(record.index))
    }

    pub fn insert_bytes_at(
        &mut self,
        index: StorageIndex,
        offset: u64,
        bytes: &[u8],
    ) -> Result<(), DbError> {
        let mut record = self.record(index.0)?;

        let id = self.transaction();
        self.ensure_size(&mut record, offset, bytes.len() as u64)?;
        let pos = record.value_start() + offset;
        self.data.write(pos, bytes)?;
        self.commit(id)
    }

    pub fn len(&self) -> u64 {
        self.data.len()
    }

    pub fn move_at(
        &mut self,
        index: StorageIndex,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<(), DbError> {
        let bytes = self
            .value_as_bytes_at_size(index, offset_from, size)?
            .to_vec();
        let id = self.transaction();
        self.insert_bytes_at(index, offset_to, &bytes)?;
        let record = self.record(index.0)?;
        self.erase_bytes(record.value_start(), offset_from, offset_to, size)?;
        self.commit(id)
    }

    pub fn name(&self) -> &str {
        self.data.name()
    }

    pub fn remove(&mut self, index: StorageIndex) -> Result<(), DbError> {
        let record = self.record(index.0)?;

        let id = self.transaction();
        self.remove_index(index.0);
        self.invalidate_record(record.pos)?;
        self.commit(id)
    }

    pub fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
        self.data.rename(new_name)
    }

    #[allow(dead_code)]
    pub fn replace<T: Serialize>(&mut self, index: StorageIndex, value: &T) -> Result<(), DbError> {
        self.replace_with_bytes(index, &value.serialize())
    }

    pub fn replace_with_bytes(&mut self, index: StorageIndex, bytes: &[u8]) -> Result<(), DbError> {
        let id = self.transaction();
        self.insert_bytes_at(index, 0, bytes)?;
        self.resize_value(index, bytes.len() as u64)?;
        self.commit(id)
    }

    #[allow(clippy::comparison_chain)]
    pub fn resize_value(&mut self, index: StorageIndex, new_size: u64) -> Result<(), DbError> {
        let mut record = self.record(index.0)?;

        let id = self.transaction();

        if new_size > record.size {
            self.enlarge_value(&mut record, new_size)?;
        } else if new_size < record.size {
            self.shrink_value(&mut record, new_size)?;
        }

        self.commit(id)
    }

    pub fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        let id = self.transaction();
        let records = self.records();
        let size = self.shrink_records(records)?;
        self.truncate(size)?;

        self.commit(id)
    }

    pub fn transaction(&mut self) -> u64 {
        self.begin_transaction()
    }

    pub fn value<T: Serialize>(&self, index: StorageIndex) -> Result<T, DbError> {
        T::deserialize(&self.value_as_bytes(index)?)
    }

    pub fn value_as_bytes(&self, index: StorageIndex) -> Result<StorageSlice, DbError> {
        self.value_as_bytes_at(index, 0)
    }

    pub fn value_as_bytes_at(
        &self,
        index: StorageIndex,
        offset: u64,
    ) -> Result<StorageSlice, DbError> {
        let size = self.value_size(index)?;
        self.value_as_bytes_at_size(index, offset, size - std::cmp::min(size, offset))
    }

    pub fn value_as_bytes_at_size(
        &self,
        index: StorageIndex,
        offset: u64,
        size: u64,
    ) -> Result<StorageSlice, DbError> {
        let record = self.record(index.0)?;
        Self::validate_read_size(offset, size, record.size)?;
        let pos = record.value_start() + offset;

        self.data.read(pos, size)
    }

    #[allow(dead_code)]
    pub fn value_at<T: Serialize>(&self, index: StorageIndex, offset: u64) -> Result<T, DbError> {
        T::deserialize(&self.value_as_bytes_at(index, offset)?)
    }

    pub fn value_size(&self, index: StorageIndex) -> Result<u64, DbError> {
        Ok(self.record(index.0)?.size)
    }

    fn append(&mut self, bytes: &[u8]) -> Result<(), DbError> {
        let len = self.len();
        self.data.write(len, bytes)
    }

    fn begin_transaction(&mut self) -> u64 {
        self.transactions += 1;
        self.transactions
    }

    fn end_transaction(&mut self, id: u64) -> Result<(), DbError> {
        if self.transactions != id {
            return Err(DbError::from(format!(
                "Cannot end transaction '{id}'. Transaction '{}' in progress.",
                self.transactions
            )));
        }

        if self.transactions != 0 {
            self.transactions -= 1;

            if self.transactions == 0 {
                self.data.flush()?;
            }
        }

        Ok(())
    }

    fn enlarge_value(&mut self, record: &mut StorageRecord, new_size: u64) -> Result<(), DbError> {
        if self.is_at_end(record)? {
            self.enlarge_at_end(record, new_size)
        } else {
            self.move_to_end(record, new_size)
        }
    }

    fn enlarge_at_end(&mut self, record: &mut StorageRecord, new_size: u64) -> Result<(), DbError> {
        let old_size = record.size;
        record.size = new_size;
        self.set_size(record.index, new_size);
        self.data.write(
            record.pos + record.index.serialized_size(),
            &record.size.serialize(),
        )?;
        self.append(&vec![0_u8; (new_size - old_size) as usize])
    }

    fn ensure_size(
        &mut self,
        record: &mut StorageRecord,
        offset: u64,
        size: u64,
    ) -> Result<(), DbError> {
        let new_size = offset + size;

        if new_size > record.size {
            self.enlarge_value(record, new_size)?;
        }

        Ok(())
    }

    #[allow(clippy::comparison_chain)]
    fn erase_bytes(
        &mut self,
        pos: u64,
        offset_from: u64,
        offset_to: u64,
        size: u64,
    ) -> Result<(), DbError> {
        if offset_from < offset_to {
            self.data.write(
                pos + offset_from,
                &vec![0_u8; std::cmp::min(size, offset_to - offset_from) as usize],
            )?;
        } else if offset_from > offset_to {
            let position = std::cmp::max(offset_to + size, offset_from);
            self.data.write(
                pos + position,
                &vec![0_u8; (offset_from + size - position) as usize],
            )?;
        }

        Ok(())
    }

    fn invalidate_record(&mut self, pos: u64) -> Result<(), DbError> {
        self.data.write(pos, &0_u64.serialize())
    }

    fn is_at_end(&mut self, record: &StorageRecord) -> Result<bool, DbError> {
        Ok(self.len() == record.end())
    }

    fn move_to_end(&mut self, record: &mut StorageRecord, new_size: u64) -> Result<(), DbError> {
        let mut bytes = self.read_value(record)?.to_vec();
        bytes.resize(new_size as usize, 0_u8);

        let len = self.len();
        self.update_record(record, len, new_size)?;
        self.append(&bytes)
    }

    fn new_record(&mut self, pos: u64, value_len: u64) -> StorageRecord {
        self.records.new_record(pos, value_len)
    }

    fn read_record(&mut self, pos: u64) -> Result<StorageRecord, DbError> {
        let bytes = self.data.read(pos, Self::record_serialized_size())?;
        let index = u64::deserialize(&bytes)?;
        let size = u64::deserialize(&bytes[index.serialized_size() as usize..])?;

        Ok(StorageRecord { index, pos, size })
    }

    fn read_records(&mut self) -> Result<(), DbError> {
        let mut records: Vec<StorageRecord> = vec![StorageRecord::default()];
        let end = self.len();
        let mut current_pos = 0;

        while current_pos < end {
            let record = self.read_record(current_pos)?;

            if record.index != 0 {
                let index = record.index as usize;

                if records.len() <= index {
                    records.resize(index + 1, StorageRecord::default());
                }

                records[index] = record;
            }

            current_pos = record.end();
        }

        self.records.set_records(records);

        Ok(())
    }

    fn read_value(&mut self, record: &StorageRecord) -> Result<StorageSlice, DbError> {
        self.data.read(record.value_start(), record.size)
    }

    fn record(&self, index: u64) -> Result<StorageRecord, DbError> {
        self.records.record(index)
    }

    fn record_serialized_size() -> u64 {
        u64::serialized_size_static() * 2
    }

    fn records(&self) -> Vec<StorageRecord> {
        self.records.records()
    }

    fn remove_index(&mut self, index: u64) {
        self.records.remove_index(index);
    }

    fn set_pos(&mut self, index: u64, pos: u64) {
        self.records.set_pos(index, pos);
    }

    fn set_size(&mut self, index: u64, size: u64) {
        self.records.set_size(index, size);
    }

    fn shrink_index(&mut self, record: &StorageRecord, current_pos: u64) -> Result<u64, DbError> {
        if record.pos != current_pos {
            let bytes = self.read_value(record)?.to_vec();
            self.set_pos(record.index, current_pos);
            self.write_record(&StorageRecord {
                index: record.index,
                pos: current_pos,
                size: record.size,
            })?;
            self.data
                .write(current_pos + Self::record_serialized_size(), &bytes)?;
        }

        Ok(current_pos + Self::record_serialized_size() + record.size)
    }

    fn shrink_records(&mut self, records: Vec<StorageRecord>) -> Result<u64, DbError> {
        let mut current_pos = 0_u64;

        for record in records {
            current_pos = self.shrink_index(&record, current_pos)?;
        }

        Ok(current_pos)
    }

    fn shrink_value(&mut self, record: &mut StorageRecord, new_size: u64) -> Result<(), DbError> {
        if self.is_at_end(record)? {
            record.size = new_size;
            self.set_size(record.index, new_size);
            self.truncate(record.end())
        } else {
            record.size = new_size;
            self.set_size(record.index, new_size);
            self.move_to_end(record, new_size)
        }
    }

    fn truncate(&mut self, size: u64) -> Result<(), DbError> {
        let current_size = self.len();

        if size < current_size {
            self.data.resize(size)?;
        }

        Ok(())
    }

    fn update_record(
        &mut self,
        record: &mut StorageRecord,
        new_pos: u64,
        new_size: u64,
    ) -> Result<(), DbError> {
        self.invalidate_record(record.pos)?;
        self.set_pos(record.index, new_pos);
        self.set_size(record.index, new_size);
        record.pos = new_pos;
        record.size = new_size;
        self.write_record(record)
    }

    fn validate_read_size(offset: u64, read_size: u64, value_size: u64) -> Result<(), DbError> {
        if offset > value_size {
            return Err(DbError::from(format!(
                "Storage error: offset ({offset}) out of bounds ({value_size})"
            )));
        }

        if (offset + read_size) > value_size {
            return Err(DbError::from(format!(
                "Storage error: value size ({}) out of bounds ({})",
                offset + read_size,
                value_size
            )));
        }

        Ok(())
    }

    fn write_record(&mut self, record: &StorageRecord) -> Result<(), DbError> {
        let mut bytes = Vec::with_capacity(Self::record_serialized_size() as usize);
        bytes.extend(record.index.serialize());
        bytes.extend(record.size.serialize());

        self.data.write(record.pos, &bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn derived_from_clone() {
        let index = StorageIndex::default();
        let other = index.clone();
        assert_eq!(index, other);
    }

    #[test]
    fn derived_from_debug() {
        format!("{:?}", StorageIndex::default());
    }

    #[test]
    fn derived_from_ord() {
        assert_eq!(
            StorageIndex::default().cmp(&StorageIndex::default()),
            Ordering::Equal
        );
    }

    #[test]
    fn derived_from_partial_ord() {
        let mut indexes = vec![
            StorageIndex::default(),
            StorageIndex::from(100_u64),
            StorageIndex::from(u64::MAX),
            StorageIndex::from(1_u64),
        ];
        indexes.sort();
        assert_eq!(
            indexes,
            vec![
                StorageIndex::default(),
                StorageIndex::from(1_u64),
                StorageIndex::from(100_u64),
                StorageIndex::from(u64::MAX),
            ]
        )
    }

    #[test]
    fn serialize() {
        let index = StorageIndex::from(1_u64);
        let bytes = index.serialize();
        let other = StorageIndex::deserialize(&bytes).unwrap();
        assert_eq!(index, other);
    }
}
