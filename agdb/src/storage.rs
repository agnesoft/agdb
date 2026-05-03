pub mod any_storage;
pub mod file_storage;
pub mod file_storage_memory_mapped;
pub mod memory_storage;

mod storage_records;
mod write_ahead_log;

use self::storage_records::StorageRecord;
use self::storage_records::StorageRecords;
use crate::DbError;
use crate::collections::vec::VecValue;
use crate::storage::storage_records::STORAGE_RECORD_SIZE;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use std::borrow::Cow;

const CURRENT_VERSION: u64 = 1;
const CHUNK_SIZE: u64 = 1024 * 1024;

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

impl<S: StorageData> VecValue<S> for StorageIndex {
    fn store(&self, _storage: &mut Storage<S>) -> Result<Vec<u8>, DbError> {
        Ok(self.serialize())
    }

    fn load(_storage: &Storage<S>, bytes: &[u8]) -> Result<Self, DbError> {
        Self::deserialize(bytes)
    }

    fn remove(_storage: &mut Storage<S>, _bytes: &[u8]) -> Result<(), DbError> {
        Ok(())
    }

    fn storage_len() -> u64 {
        StorageIndex::serialized_size_static()
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
    fn backup(&self, _name: &str) -> Result<(), DbError>;

    /// Copies the storage to a new `name`.
    fn copy(&self, name: &str) -> Result<Self, DbError>;

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
    fn read(&'_ self, pos: u64, value_len: u64) -> Result<StorageSlice<'_>, DbError>;

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

#[derive(Debug)]
pub(crate) struct Storage<D: StorageData> {
    data: D,
    records: StorageRecords,
    transactions: u64,
    version: u64,
}

impl<D: StorageData> Storage<D> {
    pub fn new(name: &str) -> Result<Self, DbError> {
        Self::with_data(D::new(name)?)
    }

    pub fn with_data(data: D) -> Result<Self, DbError> {
        let mut s = Self {
            data,
            records: StorageRecords::new(),
            transactions: 0,
            version: 0,
        };

        s.read_records()?;

        Ok(s)
    }

    pub fn backup(&self, name: &str) -> Result<(), DbError> {
        self.data.backup(name)
    }

    pub fn commit(&mut self, id: u64) -> Result<(), DbError> {
        self.end_transaction(id)
    }

    pub fn copy(&self, name: &str) -> Result<Self, DbError> {
        Ok(Self {
            data: self.data.copy(name)?,
            records: self.records.clone(),
            transactions: 0,
            version: self.version,
        })
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
        if let Some((free_pos, free_size)) = self.records.take_free(bytes.len() as u64) {
            let record = self.records.new_record(free_pos, bytes.len() as u64);
            let id = self.transaction();
            self.write_record(&record)?;
            self.data.write(record.value_start(), bytes)?;

            if free_size > bytes.len() as u64 {
                self.free_a_region(
                    record.end(),
                    free_size - STORAGE_RECORD_SIZE - bytes.len() as u64,
                )?;
            }

            self.commit(id)?;
            return Ok(StorageIndex::from(record.index));
        }

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

        if self.is_at_end(&record) {
            self.truncate(record.pos)?;
        } else {
            self.free_a_region(record.pos, record.size)?;
        }

        self.commit(id)
    }

    pub fn rename(&mut self, new_name: &str) -> Result<(), DbError> {
        self.data.rename(new_name)
    }

    pub fn replace<T: Serialize>(&mut self, index: StorageIndex, value: &T) -> Result<(), DbError> {
        self.replace_with_bytes(index, &value.serialize())
    }

    pub fn replace_with_bytes(&mut self, index: StorageIndex, bytes: &[u8]) -> Result<(), DbError> {
        let id = self.transaction();
        self.insert_bytes_at(index, 0, bytes)?;
        self.resize_value(index, bytes.len() as u64)?;
        self.commit(id)
    }

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

    fn shrink_index(
        &mut self,
        mut record: StorageRecord,
        current_pos: u64,
    ) -> Result<u64, DbError> {
        if record.pos != current_pos {
            let bytes = self.read_value(&record)?.to_vec();
            record.pos = current_pos;
            self.records.set_pos(record.index, current_pos);
            self.write_record(&record)?;
            self.data.write(current_pos + STORAGE_RECORD_SIZE, &bytes)?;
        }

        Ok(current_pos + STORAGE_RECORD_SIZE + record.size)
    }

    pub fn optimize_storage(&mut self) -> Result<(), DbError> {
        let id = self.transaction();
        let mut current_pos = Self::current_version_record().end();

        for record in self.records.records() {
            current_pos = self.shrink_index(record, current_pos)?;
        }

        self.truncate(current_pos)?;
        self.records.clear_free();
        self.commit(id)
    }

    pub fn transaction(&mut self) -> u64 {
        self.begin_transaction()
    }

    pub fn value<T: Serialize>(&self, index: StorageIndex) -> Result<T, DbError> {
        T::deserialize(&self.value_as_bytes(index)?)
    }

    pub fn value_as_bytes(&'_ self, index: StorageIndex) -> Result<StorageSlice<'_>, DbError> {
        self.value_as_bytes_at(index, 0)
    }

    pub fn value_as_bytes_at(
        &'_ self,
        index: StorageIndex,
        offset: u64,
    ) -> Result<StorageSlice<'_>, DbError> {
        let size = self.value_size(index)?;
        self.value_as_bytes_at_size(index, offset, size - std::cmp::min(size, offset))
    }

    pub fn value_as_bytes_at_size(
        &'_ self,
        index: StorageIndex,
        offset: u64,
        size: u64,
    ) -> Result<StorageSlice<'_>, DbError> {
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

    #[allow(dead_code)]
    pub fn version(&self) -> u64 {
        self.version
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
        if self.is_at_end(record) {
            self.enlarge_at_end(record, new_size)
        } else if let Some((_, free_size)) = self
            .records
            .take_free_after(record.end(), new_size - record.size)
        {
            self.enlarge_in_place(record, new_size, free_size)
        } else if let Some((free_pos, free_size)) = self.records.take_free(new_size) {
            self.enlarge_move_to(record, new_size, free_pos, free_size)
        } else {
            self.move_to_end(record, new_size)
        }
    }

    fn enlarge_move_to(
        &mut self,
        record: &mut StorageRecord,
        new_size: u64,
        free_pos: u64,
        free_size: u64,
    ) -> Result<(), DbError> {
        let mut bytes = self.data.read(record.value_start(), record.size)?.to_vec();
        bytes.resize(new_size as usize, 0_u8);
        self.free_a_region(record.pos, record.size)?;
        self.update_record(record, free_pos, new_size)?;
        self.data.write(record.value_start(), &bytes)?;

        if free_size > new_size {
            self.free_a_region(record.end(), free_size - new_size - STORAGE_RECORD_SIZE)?;
        }

        Ok(())
    }

    fn free_a_region(&mut self, pos: u64, size: u64) -> Result<(), DbError> {
        let size = self.records.mark_free_compact(pos, size);

        self.write_record(&StorageRecord {
            index: 0,
            pos,
            size,
        })
    }

    fn enlarge_in_place(
        &mut self,
        record: &mut StorageRecord,
        new_size: u64,
        free_size: u64,
    ) -> Result<(), DbError> {
        let old_size = record.size;
        let old_end = record.end();
        let header_size = STORAGE_RECORD_SIZE;
        let remainder = (old_size + header_size + free_size) - new_size;
        record.size = new_size;
        self.records.set_size(record.index, new_size);
        self.data.write(
            record.pos + record.index.serialized_size(),
            &record.size.serialize(),
        )?;
        self.data
            .write(old_end, &vec![0_u8; (new_size - old_size) as usize])?;

        if remainder != 0 {
            self.free_a_region(record.end(), remainder - STORAGE_RECORD_SIZE)?;
        }

        Ok(())
    }

    fn enlarge_at_end(&mut self, record: &mut StorageRecord, new_size: u64) -> Result<(), DbError> {
        let old_size = record.size;
        record.size = new_size;
        self.records.set_size(record.index, new_size);
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

    fn extract_version(&mut self, record: &StorageRecord) -> Result<u64, DbError> {
        if record.size < u64::serialized_size_static() {
            return Err(DbError::from(format!(
                "Storage error: invalid version record size ({} < {})",
                record.size,
                u64::serialized_size_static()
            )));
        }

        let bytes = self.read_value(record)?.to_vec();
        u64::deserialize(&bytes)
    }

    fn is_at_end(&mut self, record: &StorageRecord) -> bool {
        self.len() == record.end()
    }

    fn move_to_end(&mut self, record: &mut StorageRecord, new_size: u64) -> Result<(), DbError> {
        let mut bytes = self.read_value(record)?.to_vec();
        bytes.resize(new_size as usize, 0_u8);
        let len = self.len();
        self.free_a_region(record.pos, record.size)?;
        self.update_record(record, len, new_size)?;
        self.append(&bytes)
    }

    fn new_record(&mut self, pos: u64, value_len: u64) -> StorageRecord {
        self.records.new_record(pos, value_len)
    }

    fn read_record(&mut self, pos: u64) -> Result<StorageRecord, DbError> {
        let bytes = self.data.read(pos, STORAGE_RECORD_SIZE)?;
        let index = u64::deserialize(&bytes)?;
        let size = u64::deserialize(&bytes[index.serialized_size() as usize..])?;

        Ok(StorageRecord { index, pos, size })
    }

    fn read_records(&mut self) -> Result<(), DbError> {
        if STORAGE_RECORD_SIZE <= self.len() {
            let version_record = self.read_record(0)?;

            if version_record.index == 0 {
                self.version = self.extract_version(&version_record)?;
            }
        }

        self.validate_or_update_version()?;

        let end = self.len();
        let max_records = end / STORAGE_RECORD_SIZE;
        let mut current_pos = Self::current_version_record().end();

        while current_pos < end {
            let record = self.read_record(current_pos)?;

            if record.index != 0 && record.index > max_records {
                return Err(DbError::from(format!(
                    "Storage error: invalid record index ({}) exceeds maximum ({})",
                    record.index, max_records
                )));
            }

            if (end - current_pos) < record.size {
                return Err(DbError::from(format!(
                    "Storage error: invalid record size ({}) exceeds remaining storage ({})",
                    record.size,
                    end - current_pos
                )));
            }

            self.records.set_record(record);
            current_pos = record.end();
        }

        self.records.rebuild_free_index();

        Ok(())
    }

    fn read_value(&'_ mut self, record: &StorageRecord) -> Result<StorageSlice<'_>, DbError> {
        self.data.read(record.value_start(), record.size)
    }

    fn record(&self, index: u64) -> Result<StorageRecord, DbError> {
        self.records.record(index)
    }

    fn remove_index(&mut self, index: u64) {
        self.records.remove_index(index);
    }

    fn shrink_value(&mut self, record: &mut StorageRecord, new_size: u64) -> Result<(), DbError> {
        if self.is_at_end(record) {
            record.size = new_size;
            self.records.set_size(record.index, new_size);
            self.data
                .write(record.pos + STORAGE_RECORD_SIZE, &record.size.serialize())?;
            self.truncate(record.end())
        } else {
            let free_size = record.size - new_size;

            if free_size >= STORAGE_RECORD_SIZE {
                record.size = new_size;
                self.records.set_size(record.index, new_size);
                self.data
                    .write(record.pos + STORAGE_RECORD_SIZE, &record.size.serialize())?;
                self.free_a_region(record.end(), free_size - STORAGE_RECORD_SIZE)
            } else {
                self.move_to_end(record, new_size)
            }
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
        record.pos = new_pos;
        record.size = new_size;
        self.records.set_pos(record.index, new_pos);
        self.records.set_size(record.index, new_size);
        self.write_record(record)
    }

    fn validate_or_update_version(&mut self) -> Result<(), DbError> {
        if self.version > CURRENT_VERSION {
            return Err(DbError::from(format!(
                "Storage error: db version '{}' is higher than the current version '{CURRENT_VERSION}'",
                self.version
            )));
        }

        if self.version == CURRENT_VERSION {
            return Ok(());
        }

        self.version = CURRENT_VERSION;
        let version_record = Self::current_version_record();
        let version_record_size = version_record.end();
        let len = self.data.len();
        let mut pos = len;
        let mut size;

        let transaction_id = self.transaction();
        self.data.resize(len + version_record_size)?;

        while 0 < pos {
            if CHUNK_SIZE < pos {
                size = CHUNK_SIZE;
                pos -= size;
            } else {
                size = pos;
                pos = 0;
            }
            let data = self.data.read(pos, size)?.to_vec();
            self.data.write(pos + version_record_size, &data)?;
        }

        self.write_record(&version_record)?;
        self.data
            .write(version_record.value_start(), &self.version.serialize())?;
        self.commit(transaction_id)?;

        Ok(())
    }

    fn current_version_record() -> StorageRecord {
        StorageRecord {
            index: 0,
            pos: 0,
            size: u64::serialized_size_static(),
        }
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
        let mut bytes = Vec::with_capacity(STORAGE_RECORD_SIZE as usize);
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
        let _ = format!("{:?}", StorageIndex::default());
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
