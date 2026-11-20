pub mod any_storage;
pub mod file_storage;
pub mod file_storage_memory_mapped;
pub mod memory_storage;

mod overlay;
mod ranges;
mod storage_records;
mod write_ahead_log;

use self::storage_records::StorageRecord;
use self::storage_records::StorageRecords;
use crate::DbError;
use crate::DbErrorType;
use crate::collections::vec::VecValue;
use crate::storage::storage_records::STORAGE_RECORD_SIZE;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use std::borrow::Cow;

/// The version a storage without the version record is upgraded to.
const CURRENT_VERSION: u64 = 1;
/// The highest version that can be opened. The database sets it once
/// its data have the layout of that version so that older versions,
/// that open only the versions up to theirs, refuse it before changing
/// anything.
const MAX_VERSION: u64 = 2;
const CHUNK_SIZE: u64 = 1024 * 1024;

pub const DEFAULT_OVERLAY_LIMIT: u64 = 64 * 1024 * 1024;

/// Controls whether the underlying storage layer performs synchronization
/// on every storage commit. For example issuing `fsync`/`fdatasync`
/// for physical filesystems.
///
/// - `None` — no explicit sync; relies on the OS to flush dirty pages.
///   Suitable for local filesystems or when running in a replicated cluster.
/// - `Commit` — calls `sync_data()` on the WAL before writing the main file
///   and on the main file before clearing the WAL at each transaction
///   commit. The clearing is not synced, so a crash may roll back the last
///   commit. Required for correctness on network filesystems with
///   client-side write caching (e.g. WekaFS with `writecache`).
/// - `Strict` — like `Commit` and also syncs the cleared WAL so that a
///   commit survives a crash.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub enum SyncMode {
    #[default]
    None,
    Commit,
    Strict,
}

impl std::fmt::Display for SyncMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncMode::None => f.write_str("none"),
            SyncMode::Commit => f.write_str("commit"),
            SyncMode::Strict => f.write_str("strict"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct StorageConfig {
    pub sync_mode: SyncMode,
    pub overlay_limit: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            sync_mode: SyncMode::None,
            overlay_limit: DEFAULT_OVERLAY_LIMIT,
        }
    }
}

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
    /// Writes the current database state to `name` as a backup.
    /// File implementations copy the underlying file(s); the
    /// memory implementation dumps the buffer to disk.
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

    /// Sets how many bytes of a transaction the storage holds in memory
    /// before writing them ahead of the commit. The default implementation
    /// does nothing which is appropriate for in-memory storages.
    fn set_overlay_limit(&mut self, _limit: u64) {}

    /// Returns the overlay limit. The default is [`DEFAULT_OVERLAY_LIMIT`].
    fn overlay_limit(&self) -> u64 {
        DEFAULT_OVERLAY_LIMIT
    }

    /// Sets the sync mode for this storage. The default implementation
    /// does nothing which is appropriate for in-memory storages.
    fn set_sync_mode(&mut self, _mode: SyncMode) {}

    /// Returns the current sync mode. The default is [`SyncMode::None`].
    fn sync_mode(&self) -> SyncMode {
        SyncMode::None
    }

    /// Ensures all previously written data reaches durable storage (e.g.
    /// calls `fdatasync`). The default implementation does nothing which
    /// is appropriate for in-memory storages.
    fn sync(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    /// Writes the `bytes` to the underlying storage at `pos`. The implementation
    /// must handle the case where the `pos + bytes.len()` exceeds the current
    /// [`len()`](#method.len).
    fn write(&mut self, pos: u64, bytes: &[u8]) -> Result<(), DbError>;

    /// Replays the WAL in reverse to restore the pre-transaction file state,
    /// then clears the WAL and restores len.  Default is a no-op (no WAL).
    fn recover_from_wal(&mut self) -> Result<(), DbError> {
        Ok(())
    }
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

    pub fn recover(&mut self) -> Result<(), DbError> {
        self.data.recover_from_wal()?;
        self.transactions = 0;
        self.records = StorageRecords::new();
        self.read_records()
    }

    pub fn wipe_records(&mut self) {
        self.records = StorageRecords::new();
        self.transactions = 0;
    }

    pub fn copy(&self, name: &str) -> Result<Self, DbError> {
        let mut data = self.data.copy(name)?;
        data.set_sync_mode(self.data.sync_mode());
        data.set_overlay_limit(self.data.overlay_limit());

        Ok(Self {
            data,
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
            self.write_record_with_value(&record, bytes)?;

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
        self.write_record_with_value(&record, bytes)?;
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
        self.write_value_at(&mut record, offset, bytes)?;
        self.commit(id)
    }

    pub fn insert_zeroed(&mut self, size: u64) -> Result<StorageIndex, DbError> {
        let record = self.new_record(self.len(), size);
        let id = self.transaction();
        self.write_record(&record)?;
        self.data.resize(record.end())?;
        self.commit(id)?;
        Ok(StorageIndex::from(record.index))
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
        let mut record = self.record(index.0)?;
        Self::validate_read_size(offset_from, size, record.size)?;
        let bytes = self
            .data
            .read(record.value_start() + offset_from, size)?
            .to_vec();
        let id = self.transaction();
        self.write_value_at(&mut record, offset_to, &bytes)?;
        self.erase_bytes(record.value_start(), offset_from, offset_to, size)?;
        self.commit(id)
    }

    pub fn name(&self) -> &str {
        self.data.name()
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

    pub fn transaction(&mut self) -> u64 {
        self.begin_transaction()
    }

    pub fn set_overlay_limit(&mut self, limit: u64) {
        self.data.set_overlay_limit(limit);
    }

    pub fn overlay_limit(&self) -> u64 {
        self.data.overlay_limit()
    }

    pub fn set_sync_mode(&mut self, mode: SyncMode) {
        self.data.set_sync_mode(mode);
    }

    pub fn sync_mode(&self) -> SyncMode {
        self.data.sync_mode()
    }

    pub fn sync(&mut self) -> Result<(), DbError> {
        self.data.sync()
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

    pub fn set_version(&mut self, version: u64) -> Result<(), DbError> {
        let id = self.transaction();
        self.data.write(
            Self::current_version_record().value_start(),
            &version.serialize(),
        )?;
        self.commit(id)?;
        self.version = version;
        Ok(())
    }

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
            return Err(DbError::storage(
                DbErrorType::NotAllowed,
                format!(
                    "Cannot end transaction '{id}'. Transaction '{}' in progress.",
                    self.transactions
                ),
            ));
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
        let (pos, size) = self.records.mark_free_compact(pos, size);

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
            return Err(DbError::storage(
                DbErrorType::NotEnoughData,
                format!(
                    "Invalid version record size ({} < {})",
                    record.size,
                    u64::serialized_size_static()
                ),
            ));
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
        let mut index = [0_u8; 8];
        let mut size = [0_u8; 8];
        index.copy_from_slice(&bytes[..8]);
        size.copy_from_slice(&bytes[8..16]);

        Ok(StorageRecord {
            index: u64::from_le_bytes(index),
            pos,
            size: u64::from_le_bytes(size),
        })
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
        let mut current_pos = Self::current_version_record().end();

        while current_pos < end {
            let record = self.read_record(current_pos)?;

            if record.size > end - current_pos - STORAGE_RECORD_SIZE {
                return Err(DbError::storage(
                    DbErrorType::OutOfBounds,
                    format!(
                        "Invalid record size ({}) exceeds remaining storage ({})",
                        record.size,
                        end - current_pos - STORAGE_RECORD_SIZE
                    ),
                ));
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

    fn shrink_value(&mut self, record: &mut StorageRecord, new_size: u64) -> Result<(), DbError> {
        if self.is_at_end(record) {
            record.size = new_size;
            self.records.set_size(record.index, new_size);
            self.data.write(
                record.pos + record.size.serialized_size(),
                &record.size.serialize(),
            )?;
            self.truncate(record.end())
        } else {
            let free_size = record.size - new_size;

            if free_size >= STORAGE_RECORD_SIZE {
                record.size = new_size;
                self.records.set_size(record.index, new_size);
                self.data.write(
                    record.pos + record.index.serialized_size(),
                    &record.size.serialize(),
                )?;
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
        if self.version > MAX_VERSION {
            return Err(DbError::storage(
                DbErrorType::NotAllowed,
                format!(
                    "Storage version '{}' is higher than the current version '{MAX_VERSION}'",
                    self.version
                ),
            ));
        }

        if self.version >= CURRENT_VERSION {
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
            return Err(DbError::storage(
                DbErrorType::OutOfBounds,
                format!("Offset ({offset}) out of bounds ({value_size})"),
            ));
        }

        if (offset + read_size) > value_size {
            return Err(DbError::storage(
                DbErrorType::OutOfBounds,
                format!(
                    "Value size ({}) out of bounds ({value_size})",
                    offset + read_size,
                ),
            ));
        }

        Ok(())
    }

    fn write_value_at(
        &mut self,
        record: &mut StorageRecord,
        offset: u64,
        bytes: &[u8],
    ) -> Result<(), DbError> {
        self.ensure_size(record, offset, bytes.len() as u64)?;
        self.data.write(record.value_start() + offset, bytes)
    }

    fn write_record(&mut self, record: &StorageRecord) -> Result<(), DbError> {
        self.write_record_with_value(record, &[])
    }

    fn write_record_with_value(
        &mut self,
        record: &StorageRecord,
        value: &[u8],
    ) -> Result<(), DbError> {
        let mut bytes = Vec::with_capacity(STORAGE_RECORD_SIZE as usize + value.len());
        bytes.extend(record.index.to_le_bytes());
        bytes.extend(record.size.to_le_bytes());
        bytes.extend_from_slice(value);
        self.data.write(record.pos, &bytes)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::storage::file_storage::FileStorageImpl;
    use agplatform::Fs;
    use agplatform::Platform;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::io::ErrorKind;
    use std::sync::Arc;

    const NAME: &str = "/db";
    const WAL: &str = "/.db";

    type TestStorage = Storage<FileStorageImpl<TestPlatform>>;

    fn platform() -> Arc<TestPlatform> {
        Arc::new(test_platform())
    }

    fn open(platform: &Arc<TestPlatform>) -> Result<TestStorage, DbError> {
        FileStorageImpl::with_platform(platform.clone(), NAME).and_then(Storage::with_data)
    }

    fn new_storage(platform: &Arc<TestPlatform>) -> TestStorage {
        open(platform).unwrap()
    }

    fn reopen_killed(platform: &Arc<TestPlatform>) -> TestStorage {
        platform.fs().sync();
        platform.fs().crash();
        new_storage(platform)
    }

    fn bytes(storage: &TestStorage) -> Vec<u8> {
        storage.data.read_all().unwrap()
    }

    fn value(storage: &TestStorage, index: StorageIndex) -> Vec<u8> {
        storage.value_as_bytes(index).unwrap().to_vec()
    }

    fn record(index: u64, value: &[u8]) -> Vec<u8> {
        [
            &index.to_le_bytes(),
            &(value.len() as u64).to_le_bytes(),
            value,
        ]
        .concat()
    }

    fn version_record(version: u64) -> Vec<u8> {
        record(0, &version.to_le_bytes())
    }

    fn not_found(index: u64) -> DbError {
        DbError::storage(DbErrorType::NotFound, format!("Index ({index}) not found"))
    }

    /// Fails each read, write and truncation of the files the `action`
    /// does, one at a time, and checks that recovering from the failure
    /// restores the storage the `setup` prepared.
    fn assert_failures_recover(
        setup: impl Fn(&mut TestStorage) -> Vec<StorageIndex>,
        action: impl Fn(&mut TestStorage, &[StorageIndex]) -> Result<(), DbError>,
    ) {
        for (op, path) in [
            (FsOp::Read, NAME),
            (FsOp::Write, NAME),
            (FsOp::SetLen, NAME),
            (FsOp::Write, WAL),
        ] {
            for nth in 0.. {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let indexes = setup(&mut storage);
                let before = bytes(&storage);
                let mut results = vec![None; nth];
                results.push(Some(ErrorKind::Other));
                platform.fs().set_io_errors(op, path, results);

                let result = action(&mut storage, &indexes);

                platform.fs().clear_faults();

                if result.is_ok() {
                    break;
                }

                storage.recover().unwrap();
                assert_eq!(
                    bytes(&storage),
                    before,
                    "op: {op:?}, path: {path}, nth: {nth}"
                );
            }
        }
    }

    mod sync_mode {
        use super::*;

        mod std_fmt_display {
            use super::*;

            #[test]
            fn fmt_should_show_lowercase_name() {
                assert_eq!(SyncMode::None.to_string(), "none");
                assert_eq!(SyncMode::Commit.to_string(), "commit");
                assert_eq!(SyncMode::Strict.to_string(), "strict");
            }
        }
    }

    mod storage_config {
        use super::*;

        mod default {
            use super::*;

            #[test]
            fn default_should_not_sync_and_use_default_overlay_limit() {
                assert_eq!(
                    StorageConfig::default(),
                    StorageConfig {
                        sync_mode: SyncMode::None,
                        overlay_limit: DEFAULT_OVERLAY_LIMIT,
                    }
                );
            }
        }
    }

    mod storage_index {
        use super::*;

        mod from {
            use super::*;

            #[test]
            fn from_should_wrap_index() {
                assert_eq!(StorageIndex::from(3_u64), StorageIndex(3));
            }
        }

        mod serialize {
            use super::*;

            #[allow(clippy::module_inception)]
            mod serialize {
                use super::*;

                #[test]
                fn serialize_should_write_index_as_u64() {
                    assert_eq!(StorageIndex(3).serialize(), 3_u64.serialize());
                }
            }

            mod deserialize {
                use super::*;

                #[test]
                fn deserialize_should_read_serialized_index() {
                    assert_eq!(
                        StorageIndex::deserialize(&StorageIndex(3).serialize()),
                        Ok(StorageIndex(3))
                    );
                }

                #[test]
                fn deserialize_short_bytes_should_fail() {
                    assert!(StorageIndex::deserialize(&[3, 0, 0, 0, 0, 0, 0]).is_err());
                }
            }

            mod serialized_size {
                use super::*;

                #[test]
                fn serialized_size_should_be_u64_size() {
                    assert_eq!(StorageIndex(3).serialized_size(), 8);
                }
            }
        }

        mod vec_value {
            use super::*;

            mod store {
                use super::*;

                #[test]
                fn store_should_serialize_index() {
                    let platform = platform();
                    let mut storage = new_storage(&platform);

                    assert_eq!(StorageIndex(3).store(&mut storage), Ok(3_u64.serialize()));
                }
            }

            mod load {
                use super::*;

                #[test]
                fn load_should_deserialize_index() {
                    let platform = platform();
                    let storage = new_storage(&platform);

                    assert_eq!(
                        StorageIndex::load(&storage, &3_u64.serialize()),
                        Ok(StorageIndex(3))
                    );
                }
            }

            mod remove {
                use super::*;

                #[test]
                fn remove_should_leave_storage_unchanged() {
                    let platform = platform();
                    let mut storage = new_storage(&platform);
                    let before = bytes(&storage);

                    assert_eq!(
                        StorageIndex::remove(&mut storage, &3_u64.serialize()),
                        Ok(())
                    );
                    assert_eq!(bytes(&storage), before);
                }
            }

            mod storage_len {
                use super::*;

                #[test]
                fn storage_len_should_be_u64_size() {
                    assert_eq!(
                        <StorageIndex as VecValue<FileStorageImpl<TestPlatform>>>::storage_len(),
                        8
                    );
                }
            }
        }
    }

    pub(crate) mod storage_data {
        use super::*;

        pub(crate) mod backup {
            use super::*;

            pub(crate) fn backup_should_write_bytes_to_name<S: StorageData>(
                mut storage: S,
                name: &str,
                read: impl Fn(&str) -> Vec<u8>,
            ) {
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();

                storage.backup(name).unwrap();

                assert_eq!(read(name), vec![1, 2, 3]);
            }
        }

        pub(crate) mod copy {
            use super::*;

            pub(crate) fn copy_should_create_independent_storage_named_name<S: StorageData>(
                mut storage: S,
                name: &str,
            ) {
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();

                let mut copy = storage.copy(name).unwrap();

                assert_eq!(copy.name(), name);
                assert_eq!(copy.read(0, 3).unwrap().to_vec(), vec![1, 2, 3]);
                copy.write(0, &[4]).unwrap();
                assert_eq!(storage.read(0, 1).unwrap().to_vec(), vec![1]);
            }
        }

        pub(crate) mod flush {
            use super::*;

            pub(crate) fn flush_should_keep_written_bytes_on_recover<S: StorageData>(
                mut storage: S,
            ) {
                storage.write(0, &[1, 2, 3]).unwrap();

                storage.flush().unwrap();

                storage.recover_from_wal().unwrap();
                assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![1, 2, 3]);
            }
        }

        pub(crate) mod is_empty {
            use super::*;

            pub(crate) fn is_empty_should_be_true_only_without_bytes<S: StorageData>(
                mut storage: S,
            ) {
                let empty = storage.is_empty();

                storage.write(0, &[1]).unwrap();

                assert!(empty);
                assert!(!storage.is_empty());
            }
        }

        pub(crate) mod len {
            use super::*;

            pub(crate) fn len_should_cover_written_bytes<S: StorageData>(mut storage: S) {
                storage.write(3, &[1, 2]).unwrap();

                assert_eq!(storage.len(), 5);
            }
        }

        pub(crate) mod name {
            use super::*;

            pub(crate) fn name_should_return_name<S: StorageData>(storage: S, name: &str) {
                assert_eq!(storage.name(), name);
            }
        }

        pub(crate) mod new {
            use super::*;

            pub(crate) fn new_should_create_empty_storage_named_name<S: StorageData>() {
                let storage = S::new("/new").unwrap();

                assert_eq!(storage.name(), "/new");
                assert_eq!(storage.len(), 0);
            }
        }

        pub(crate) mod read {
            use super::*;

            pub(crate) fn read_should_return_bytes_at_pos<S: StorageData>(mut storage: S) {
                storage.write(0, &[1, 2, 3, 4]).unwrap();

                assert_eq!(storage.read(1, 2).unwrap().to_vec(), vec![2, 3]);
                assert!(storage.read(4, 0).unwrap().is_empty());
            }

            pub(crate) fn read_out_of_bounds_should_fail<S: StorageData>(mut storage: S) {
                storage.write(0, &[1, 2, 3, 4]).unwrap();

                for (pos, len) in [(3, 2), (u64::MAX, 1)] {
                    assert!(storage.read(pos, len).is_err(), "pos: {pos}, len: {len}");
                }
            }
        }

        pub(crate) mod rename {
            use super::*;

            pub(crate) fn rename_should_change_name_and_keep_bytes<S: StorageData>(
                mut storage: S,
                new_name: &str,
            ) {
                storage.write(0, &[1, 2]).unwrap();
                storage.flush().unwrap();

                storage.rename(new_name).unwrap();

                assert_eq!(storage.name(), new_name);
                assert_eq!(storage.read(0, 2).unwrap().to_vec(), vec![1, 2]);
            }
        }

        pub(crate) mod resize {
            use super::*;

            pub(crate) fn resize_should_zero_new_bytes_and_cut_off_others<S: StorageData>(
                mut storage: S,
            ) {
                storage.write(0, &[1, 2, 3]).unwrap();

                storage.resize(5).unwrap();
                let enlarged = storage.read(0, 5).unwrap().to_vec();
                storage.resize(5).unwrap();
                storage.resize(2).unwrap();

                assert_eq!(enlarged, vec![1, 2, 3, 0, 0]);
                assert_eq!(storage.len(), 2);
                assert_eq!(storage.read(0, 2).unwrap().to_vec(), vec![1, 2]);
            }
        }

        pub(crate) mod set_overlay_limit {
            use super::*;

            pub(crate) fn set_overlay_limit_should_set_limit_if_supported<S: StorageData>(
                mut storage: S,
                supported: bool,
            ) {
                storage.set_overlay_limit(5);

                let expected = if supported { 5 } else { DEFAULT_OVERLAY_LIMIT };
                assert_eq!(storage.overlay_limit(), expected);
            }
        }

        pub(crate) mod overlay_limit {
            use super::*;

            pub(crate) fn overlay_limit_should_default_to_default_overlay_limit<S: StorageData>(
                storage: S,
            ) {
                assert_eq!(storage.overlay_limit(), DEFAULT_OVERLAY_LIMIT);
            }
        }

        pub(crate) mod set_sync_mode {
            use super::*;

            pub(crate) fn set_sync_mode_should_set_mode_if_supported<S: StorageData>(
                mut storage: S,
                supported: bool,
            ) {
                storage.set_sync_mode(SyncMode::Commit);

                let expected = if supported {
                    SyncMode::Commit
                } else {
                    SyncMode::None
                };
                assert_eq!(storage.sync_mode(), expected);
            }
        }

        pub(crate) mod sync_mode {
            use super::*;

            pub(crate) fn sync_mode_should_default_to_none<S: StorageData>(storage: S) {
                assert_eq!(storage.sync_mode(), SyncMode::None);
            }
        }

        pub(crate) mod sync {
            use super::*;

            pub(crate) fn sync_should_succeed<S: StorageData>(mut storage: S) {
                storage.write(0, &[1]).unwrap();

                storage.sync().unwrap();
            }
        }

        pub(crate) mod write {
            use super::*;

            pub(crate) fn write_should_overwrite_and_extend<S: StorageData>(mut storage: S) {
                storage.write(0, &[1, 2, 3]).unwrap();

                storage.write(2, &[4, 5]).unwrap();
                storage.write(0, &[6]).unwrap();

                assert_eq!(storage.len(), 4);
                assert_eq!(storage.read(0, 4).unwrap().to_vec(), vec![6, 2, 4, 5]);
            }

            pub(crate) fn write_past_end_should_zero_gap<S: StorageData>(mut storage: S) {
                storage.write(2, &[1]).unwrap();

                assert_eq!(storage.read(0, 3).unwrap().to_vec(), vec![0, 0, 1]);
            }

            pub(crate) fn write_empty_should_do_nothing<S: StorageData>(mut storage: S) {
                storage.write(5, &[]).unwrap();

                assert_eq!(storage.len(), 0);
            }
        }

        pub(crate) mod recover_from_wal {
            use super::*;

            pub(crate) fn recover_from_wal_should_restore_flushed_bytes_if_logged<
                S: StorageData,
            >(
                mut storage: S,
                logged: bool,
            ) {
                storage.write(0, &[1, 2, 3]).unwrap();
                storage.flush().unwrap();
                storage.write(1, &[4, 5, 6]).unwrap();

                storage.recover_from_wal().unwrap();

                let (len, expected) = if logged {
                    (3, vec![1, 2, 3])
                } else {
                    (4, vec![1, 4, 5, 6])
                };
                assert_eq!(storage.len(), len);
                assert_eq!(storage.read(0, len).unwrap().to_vec(), expected);
            }

            fn bytes<S: StorageData>(storage: &S) -> Vec<u8> {
                if storage.is_empty() {
                    return Vec::new();
                }

                storage.read(0, storage.len()).unwrap().to_vec()
            }

            pub(crate) fn recover_from_wal_should_restore_flushed_bytes_after_random_changes<
                S: StorageData,
            >(
                mut open: impl FnMut() -> S,
                kill: impl Fn(),
                fault: impl Fn(u64),
            ) {
                let mut seed = 0x9E37_79B9_7F4A_7C15_u64;
                let mut random = move |n: u64| {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    seed % n
                };
                let mut storage = open();
                let mut flushed = Vec::new();
                let mut expected = Vec::new();

                for round in 0..500 {
                    storage
                        .set_overlay_limit([0, 64, 512, DEFAULT_OVERLAY_LIMIT][random(4) as usize]);
                    storage.set_sync_mode(
                        [SyncMode::None, SyncMode::Commit, SyncMode::Strict][random(3) as usize],
                    );

                    for _ in 0..1 + random(8) {
                        if random(4) == 0 {
                            let len = random(64);
                            storage.resize(len).unwrap();
                            expected.resize(len as usize, 0);
                        } else {
                            let pos = random(expected.len() as u64 + 8) as usize;
                            let value = vec![1 + random(255) as u8; 1 + random(16) as usize];
                            storage.write(pos as u64, &value).unwrap();
                            expected.resize(expected.len().max(pos + value.len()), 0);
                            expected[pos..pos + value.len()].copy_from_slice(&value);
                        }
                    }

                    assert_eq!(bytes(&storage), expected, "round: {round}");

                    match random(4) {
                        0 => {
                            storage.flush().unwrap();
                            flushed = expected.clone();
                        }
                        1 => {
                            storage.recover_from_wal().unwrap();
                            expected = flushed.clone();
                        }
                        2 => {
                            kill();
                            storage = open();
                            expected = flushed.clone();
                        }
                        _ => {
                            fault(random(u64::MAX));

                            if storage.flush().is_ok() {
                                flushed = expected.clone();
                            }

                            kill();
                            storage = open();
                            expected = flushed.clone();
                        }
                    }

                    assert_eq!(bytes(&storage), expected, "round: {round}");
                }
            }
        }
    }

    mod storage {
        use super::*;

        #[test]
        fn storage_should_load_committed_values_after_reopen() {
            let platform = platform();
            let mut storage = new_storage(&platform);
            let index1 = storage.insert(&"Hello, World!".to_string()).unwrap();
            let index2 = storage.insert(&10_i64).unwrap();
            let index3 = storage.insert(&vec![1_u64, 2, 3]).unwrap();
            storage.remove(index2).unwrap();
            drop(storage);

            let mut storage = new_storage(&platform);

            assert_eq!(storage.value(index1), Ok("Hello, World!".to_string()));
            assert_eq!(storage.value::<i64>(index2), Err(not_found(2)));
            assert_eq!(storage.value(index3), Ok(vec![1_u64, 2, 3]));
            assert_eq!(storage.insert(&1_i64), Ok(index2));
        }

        #[test]
        fn storage_should_roll_back_unfinished_transaction_on_reopen() {
            for killed in [false, true] {
                // GIVEN a committed value and a transaction changing it
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&1_i64).unwrap();
                let committed = bytes(&storage);
                storage.transaction();
                storage.replace(index, &vec![1_u64; 10]).unwrap();
                storage.insert(&2_i64).unwrap();

                // WHEN the storage is reopened without the commit
                let storage = if killed {
                    reopen_killed(&platform)
                } else {
                    drop(storage);
                    new_storage(&platform)
                };

                // THEN it is as committed
                assert_eq!(bytes(&storage), committed, "killed: {killed}");
                assert_eq!(storage.value(index), Ok(1_i64), "killed: {killed}");
            }
        }

        mod new {
            use super::*;

            #[test]
            fn new_should_create_storage_with_version_record() {
                let storage = TestStorage::new(NAME).unwrap();

                assert_eq!(storage.version(), CURRENT_VERSION);
                assert_eq!(bytes(&storage), version_record(CURRENT_VERSION));
            }

            #[test]
            fn new_data_failure_should_fail() {
                assert!(TestStorage::new("/missing/db").is_err());
            }

            #[test]
            fn new_without_version_should_add_version_record() {
                for value in [vec![1_u8; 16], vec![2_u8; CHUNK_SIZE as usize]] {
                    let platform = platform();
                    platform
                        .fs()
                        .write(NAME, [record(1, &value), record(2, &[3])].concat())
                        .unwrap();

                    let storage = new_storage(&platform);

                    assert_eq!(storage.version(), CURRENT_VERSION, "len: {}", value.len());
                    assert_eq!(
                        storage.value_as_bytes(1.into()).unwrap().to_vec(),
                        value,
                        "len: {}",
                        value.len()
                    );
                    assert_eq!(
                        storage.value_as_bytes(2.into()).unwrap().to_vec(),
                        vec![3],
                        "len: {}",
                        value.len()
                    );
                }
            }

            #[test]
            fn new_version_update_failure_should_keep_storage_loadable() {
                let file = [record(1, &[1, 2]), record(2, &[3])].concat();

                for (op, path) in [
                    (FsOp::Read, NAME),
                    (FsOp::Write, NAME),
                    (FsOp::SetLen, NAME),
                    (FsOp::Write, WAL),
                ] {
                    for nth in 0.. {
                        let platform = platform();
                        platform.fs().write(NAME, &file).unwrap();
                        let mut results = vec![None; nth];
                        results.push(Some(ErrorKind::Other));
                        platform.fs().set_io_errors(op, path, results);

                        let result = open(&platform);

                        platform.fs().clear_faults();

                        if result.is_ok() {
                            break;
                        }

                        let storage = new_storage(&platform);
                        let case = format!("op: {op:?}, path: {path}, nth: {nth}");
                        assert_eq!(storage.version(), CURRENT_VERSION, "{case}");
                        assert_eq!(value(&storage, 1.into()), vec![1, 2], "{case}");
                        assert_eq!(value(&storage, 2.into()), vec![3], "{case}");
                    }
                }
            }

            #[test]
            fn new_read_failure_should_fail() {
                for nth in 0.. {
                    let platform = platform();
                    platform
                        .fs()
                        .write(
                            NAME,
                            [version_record(CURRENT_VERSION), record(1, &[1])].concat(),
                        )
                        .unwrap();
                    let mut results = vec![None; nth];
                    results.push(Some(ErrorKind::Other));
                    platform.fs().set_io_errors(FsOp::Read, NAME, results);

                    if open(&platform).is_ok() {
                        assert!(nth > 0);
                        break;
                    }
                }
            }

            #[test]
            fn new_with_higher_version_should_fail() {
                let platform = platform();
                platform
                    .fs()
                    .write(NAME, version_record(MAX_VERSION + 1))
                    .unwrap();

                assert_eq!(
                    open(&platform).unwrap_err(),
                    DbError::storage(
                        DbErrorType::NotAllowed,
                        "Storage version '3' is higher than the current version '2'"
                    )
                );
            }

            #[test]
            fn new_with_max_version_should_keep_it() {
                let platform = platform();
                platform
                    .fs()
                    .write(
                        NAME,
                        [version_record(MAX_VERSION), record(1, &[1])].concat(),
                    )
                    .unwrap();

                let storage = new_storage(&platform);

                assert_eq!(storage.version(), MAX_VERSION);
                assert_eq!(
                    bytes(&storage),
                    [version_record(MAX_VERSION), record(1, &[1])].concat()
                );
            }

            #[test]
            fn new_with_short_version_record_should_fail() {
                let platform = platform();
                platform.fs().write(NAME, record(0, &[1, 0, 0, 0])).unwrap();

                assert_eq!(
                    open(&platform).unwrap_err(),
                    DbError::storage(
                        DbErrorType::NotEnoughData,
                        "Invalid version record size (4 < 8)"
                    )
                );
            }

            #[test]
            fn new_with_record_past_end_should_fail() {
                let mut past_end = record(1, &[1; 8]);
                past_end[8] = 9;

                for invalid in [past_end, record(1, &[1; 8])[..20].to_vec(), vec![1; 4]] {
                    let platform = platform();
                    platform
                        .fs()
                        .write(NAME, [version_record(1), invalid.clone()].concat())
                        .unwrap();

                    assert!(open(&platform).is_err(), "invalid: {invalid:?}");
                }
            }

            #[test]
            fn new_with_record_past_end_should_report_remaining_size() {
                let mut past_end = record(1, &[1; 8]);
                past_end[8] = 9;
                let platform = platform();
                platform
                    .fs()
                    .write(NAME, [version_record(1), past_end].concat())
                    .unwrap();

                assert_eq!(
                    open(&platform).unwrap_err(),
                    DbError::storage(
                        DbErrorType::OutOfBounds,
                        "Invalid record size (9) exceeds remaining storage (8)"
                    )
                );
            }

            #[test]
            fn new_should_load_free_regions() {
                let platform = platform();
                platform
                    .fs()
                    .write(
                        NAME,
                        [version_record(1), record(0, &[0; 8]), record(1, &[1])].concat(),
                    )
                    .unwrap();

                let mut storage = new_storage(&platform);

                assert_eq!(storage.records.free_size(), 8);
                assert_eq!(storage.insert_bytes(&[2; 8]), Ok(2.into()));
                assert_eq!(storage.records.free_size(), 0);
            }
        }

        mod backup {
            use super::*;

            #[test]
            fn backup_should_write_storage_to_name() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&1_i64).unwrap();

                storage.backup("/backup").unwrap();

                let backup = Storage::with_data(
                    FileStorageImpl::with_platform(platform, "/backup").unwrap(),
                )
                .unwrap();
                assert_eq!(backup.value(index), Ok(1_i64));
            }
        }

        mod commit {
            use super::*;

            #[test]
            fn commit_should_end_transaction_making_changes_permanent() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let id = storage.transaction();
                let index = storage.insert(&1_i64).unwrap();

                storage.commit(id).unwrap();

                storage.recover().unwrap();
                assert_eq!(storage.value(index), Ok(1_i64));
            }

            #[test]
            fn commit_nested_should_leave_changes_to_outer_transaction() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                storage.transaction();
                let id = storage.transaction();
                let index = storage.insert(&1_i64).unwrap();

                storage.commit(id).unwrap();

                storage.recover().unwrap();
                assert_eq!(storage.value::<i64>(index), Err(not_found(1)));
            }

            #[test]
            fn commit_without_transaction_should_do_nothing() {
                let platform = platform();
                let mut storage = new_storage(&platform);

                assert_eq!(storage.commit(0), Ok(()));
            }

            #[test]
            fn commit_of_outer_transaction_should_fail() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let id = storage.transaction();
                storage.transaction();

                assert_eq!(
                    storage.commit(id),
                    Err(DbError::storage(
                        DbErrorType::NotAllowed,
                        "Cannot end transaction '1'. Transaction '2' in progress."
                    ))
                );
            }
        }

        mod recover {
            use super::*;

            #[test]
            fn recover_should_restore_last_commit() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&1_i64).unwrap();
                storage.transaction();
                storage.replace(index, &2_i64).unwrap();
                storage.insert(&3_i64).unwrap();

                storage.recover().unwrap();

                assert_eq!(storage.value(index), Ok(1_i64));
                assert_eq!(storage.value::<i64>(2.into()), Err(not_found(2)));
                assert_eq!(storage.transaction(), 1);
            }

            #[test]
            fn recover_failure_should_fail() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                storage.set_overlay_limit(0);
                storage.transaction();
                storage.insert(&1_i64).unwrap();
                platform
                    .fs()
                    .set_io_error(FsOp::Read, WAL, ErrorKind::Other);

                assert!(storage.recover().is_err());
            }
        }

        mod wipe_records {
            use super::*;

            #[test]
            fn wipe_records_should_forget_records_and_transactions() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&1_i64).unwrap();
                storage.transaction();

                storage.wipe_records();

                assert_eq!(storage.value::<i64>(index), Err(not_found(1)));
                assert_eq!(storage.transaction(), 1);
            }
        }

        mod copy {
            use super::*;

            #[test]
            fn copy_should_copy_values_to_name() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&1_i64).unwrap();

                let mut copy = storage.copy("/copy").unwrap();

                assert_eq!(copy.name(), "/copy");
                assert_eq!(copy.value(index), Ok(1_i64));
                assert_eq!(copy.insert(&2_i64), storage.insert(&3_i64));
            }

            #[test]
            fn copy_should_keep_sync_mode_and_overlay_limit() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                storage.set_sync_mode(SyncMode::Strict);
                storage.set_overlay_limit(16);

                let copy = storage.copy("/copy").unwrap();

                assert_eq!(copy.sync_mode(), SyncMode::Strict);
                assert_eq!(copy.overlay_limit(), 16);
            }

            #[test]
            fn copy_failure_should_fail() {
                let platform = platform();
                let storage = new_storage(&platform);

                assert!(storage.copy("/missing/copy").is_err());
            }
        }

        mod insert {
            use super::*;

            #[test]
            fn insert_should_store_values() {
                let platform = platform();
                let mut storage = new_storage(&platform);

                let index1 = storage.insert(&"Hello, World!".to_string()).unwrap();
                let index2 = storage.insert(&10_i64).unwrap();
                let index3 = storage.insert(&vec![1_u64, 2, 3]).unwrap();

                assert_eq!((index1, index2, index3), (1.into(), 2.into(), 3.into()));
                assert_eq!(storage.value(index1), Ok("Hello, World!".to_string()));
                assert_eq!(storage.value(index2), Ok(10_i64));
                assert_eq!(storage.value(index3), Ok(vec![1_u64, 2, 3]));
            }

            #[test]
            fn insert_should_append_record_at_end() {
                let platform = platform();
                let mut storage = new_storage(&platform);

                storage.insert(&10_i64).unwrap();

                assert_eq!(
                    bytes(&storage),
                    [version_record(1), record(1, &10_i64.to_le_bytes())].concat()
                );
            }

            #[test]
            fn insert_failure_should_be_recovered() {
                assert_failures_recover(
                    |_| vec![],
                    |storage, _| storage.insert(&1_i64).map(|_| ()),
                );
            }
        }

        mod insert_at {
            use super::*;

            #[test]
            fn insert_at_should_overwrite_value_at_offset() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&vec![1_i64, 2, 3]).unwrap();

                storage.insert_at(index, 16, &10_i64).unwrap();

                assert_eq!(storage.value(index), Ok(vec![1_i64, 10, 3]));
            }

            #[test]
            fn insert_at_past_value_end_should_enlarge_value() {
                for others in [false, true] {
                    let platform = platform();
                    let mut storage = new_storage(&platform);
                    let index = storage.insert_bytes(&[1, 2]).unwrap();
                    let other = others.then(|| storage.insert(&3_i64).unwrap());

                    storage.insert_bytes_at(index, 4, &[5, 6]).unwrap();

                    assert_eq!(
                        value(&storage, index),
                        vec![1, 2, 0, 0, 5, 6],
                        "others: {others}"
                    );
                    if let Some(other) = other {
                        assert_eq!(storage.value(other), Ok(3_i64));
                    }
                }
            }

            #[test]
            fn insert_at_missing_index_should_fail() {
                let platform = platform();
                let mut storage = new_storage(&platform);

                assert_eq!(storage.insert_at(1.into(), 0, &1_i64), Err(not_found(1)));
            }
        }

        mod insert_bytes {
            use super::*;

            #[test]
            fn insert_bytes_should_reuse_free_region_of_same_size() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                storage.insert_bytes(&[1; 8]).unwrap();
                let index = storage.insert_bytes(&[2; 8]).unwrap();
                storage.insert_bytes(&[3; 8]).unwrap();
                storage.remove(index).unwrap();
                let len = storage.len();

                let reused = storage.insert_bytes(&[4; 8]).unwrap();

                assert_eq!(reused, index);
                assert_eq!(storage.len(), len);
                assert_eq!(value(&storage, reused), vec![4; 8]);
                assert_eq!(storage.records.free_size(), 0);
            }

            #[test]
            fn insert_bytes_should_split_larger_free_region() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert_bytes(&[1; 40]).unwrap();
                let other = storage.insert_bytes(&[2; 8]).unwrap();
                storage.remove(index).unwrap();
                let len = storage.len();

                let reused = storage.insert_bytes(&[3; 8]).unwrap();

                assert_eq!(storage.len(), len);
                assert_eq!(storage.records.free_size(), 16);
                let mut storage = reopen_killed(&platform);
                assert_eq!(value(&storage, reused), vec![3; 8]);
                assert_eq!(value(&storage, other), vec![2; 8]);
                assert_eq!(storage.records.free_size(), 16);
                assert_eq!(storage.insert_bytes(&[4; 16]).unwrap(), 3.into());
                assert_eq!(storage.len(), len);
            }

            #[test]
            fn insert_bytes_should_not_split_free_region_without_room_for_record() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert_bytes(&[1; 20]).unwrap();
                storage.insert_bytes(&[2; 8]).unwrap();
                storage.remove(index).unwrap();
                let len = storage.len();

                storage.insert_bytes(&[3; 8]).unwrap();

                assert_eq!(storage.len(), len + 24);
                assert_eq!(storage.records.free_size(), 20);
            }

            #[test]
            fn insert_bytes_failure_should_be_recovered() {
                for (removed, size) in [(8, 8), (40, 8)] {
                    assert_failures_recover(
                        |storage| {
                            let index = storage.insert_bytes(&vec![1; removed]).unwrap();
                            storage.insert_bytes(&[2]).unwrap();
                            storage.remove(index).unwrap();
                            vec![]
                        },
                        |storage, _| storage.insert_bytes(&vec![3; size]).map(|_| ()),
                    );
                }
            }
        }

        mod insert_bytes_at {
            use super::*;

            #[test]
            fn insert_bytes_at_failure_should_be_recovered() {
                assert_failures_recover(
                    |storage| vec![storage.insert_bytes(&[1, 2]).unwrap()],
                    |storage, indexes| storage.insert_bytes_at(indexes[0], 1, &[3, 4]),
                );
            }
        }

        mod insert_zeroed {
            use super::*;

            #[test]
            fn insert_zeroed_should_append_zero_bytes_past_free_regions() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert_bytes(&[1; 40]).unwrap();
                storage.insert_bytes(&[2; 8]).unwrap();
                storage.remove(index).unwrap();
                let len = storage.len();

                let zeroed = storage.insert_zeroed(16).unwrap();

                assert_eq!(storage.len(), len + 32);
                assert_eq!(storage.records.free_size(), 40);
                let storage = reopen_killed(&platform);
                assert_eq!(value(&storage, zeroed), vec![0; 16]);
            }

            #[test]
            fn insert_zeroed_failure_should_be_recovered() {
                assert_failures_recover(
                    |storage| vec![storage.insert_bytes(&[1, 2]).unwrap()],
                    |storage, _| storage.insert_zeroed(16).map(|_| ()),
                );
            }
        }

        mod move_at {
            use super::*;

            #[test]
            fn move_at_missing_index_should_fail() {
                let platform = platform();
                let mut storage = new_storage(&platform);

                assert_eq!(storage.move_at(1.into(), 0, 1, 1), Err(not_found(1)));
            }

            #[test]
            fn move_at_should_move_bytes_and_zero_vacated_ones() {
                for (from, to, size, expected) in [
                    (4, 2, 2, vec![1, 2, 5, 6, 0, 0]),
                    (2, 0, 4, vec![3, 4, 5, 6, 0, 0]),
                    (0, 4, 2, vec![0, 0, 3, 4, 1, 2]),
                    (0, 2, 4, vec![0, 0, 1, 2, 3, 4]),
                    (2, 8, 2, vec![1, 2, 0, 0, 5, 6, 0, 0, 3, 4]),
                    (2, 2, 2, vec![1, 2, 3, 4, 5, 6]),
                    (0, 1, 0, vec![1, 2, 3, 4, 5, 6]),
                ] {
                    let platform = platform();
                    let mut storage = new_storage(&platform);
                    let index = storage.insert_bytes(&[1, 2, 3, 4, 5, 6]).unwrap();

                    storage.move_at(index, from, to, size).unwrap();

                    assert_eq!(
                        value(&storage, index),
                        expected,
                        "from: {from}, to: {to}, size: {size}"
                    );
                }
            }

            #[test]
            fn move_at_size_out_of_bounds_should_fail() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert_bytes(&[1, 2, 3, 4, 5, 6]).unwrap();

                assert_eq!(
                    storage.move_at(index, 2, 4, 100),
                    Err(DbError::storage(
                        DbErrorType::OutOfBounds,
                        "Value size (102) out of bounds (6)"
                    ))
                );
            }

            #[test]
            fn move_at_failure_should_be_recovered() {
                for (from, to) in [(4, 2), (0, 4), (2, 8)] {
                    assert_failures_recover(
                        |storage| vec![storage.insert_bytes(&[1, 2, 3, 4, 5, 6]).unwrap()],
                        |storage, indexes| storage.move_at(indexes[0], from, to, 2),
                    );
                }
            }
        }

        mod name {
            use super::*;

            #[test]
            fn name_should_return_name_of_data() {
                let platform = platform();
                let storage = new_storage(&platform);

                assert_eq!(storage.name(), NAME);
            }
        }

        mod optimize_storage {
            use super::*;

            #[test]
            fn optimize_storage_should_remove_free_regions() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index1 = storage.insert(&1_i64).unwrap();
                let index2 = storage.insert(&2_i64).unwrap();
                let index3 = storage.insert(&3_i64).unwrap();
                storage.remove(index2).unwrap();
                storage.resize_value(index1, 4).unwrap();

                storage.optimize_storage().unwrap();

                assert_eq!(storage.records.free_size(), 0);
                assert_eq!(
                    bytes(&storage),
                    [
                        version_record(1),
                        record(3, &3_i64.to_le_bytes()),
                        record(1, &[1, 0, 0, 0])
                    ]
                    .concat()
                );
                let storage = reopen_killed(&platform);
                assert_eq!(
                    storage.value_as_bytes(index1).unwrap().to_vec(),
                    vec![1, 0, 0, 0]
                );
                assert_eq!(storage.value(index3), Ok(3_i64));
            }

            #[test]
            fn optimize_storage_without_free_regions_should_keep_storage() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                storage.insert(&1_i64).unwrap();
                let before = bytes(&storage);

                storage.optimize_storage().unwrap();

                assert_eq!(bytes(&storage), before);
            }

            #[test]
            fn optimize_storage_failure_should_be_recovered() {
                assert_failures_recover(
                    |storage| {
                        let index = storage.insert(&1_i64).unwrap();
                        storage.insert(&2_i64).unwrap();
                        storage.remove(index).unwrap();
                        vec![]
                    },
                    |storage, _| storage.optimize_storage(),
                );
            }
        }

        mod remove {
            use super::*;

            #[test]
            fn remove_at_end_should_truncate_storage() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&1_i64).unwrap();

                storage.remove(index).unwrap();

                assert_eq!(storage.value::<i64>(index), Err(not_found(1)));
                assert_eq!(bytes(&storage), version_record(1));
            }

            #[test]
            fn remove_should_free_region_merged_with_adjacent_ones() {
                for (removed, free) in [
                    (vec![2], vec![8]),
                    (vec![2, 3], vec![32]),
                    (vec![3, 2], vec![32]),
                    (vec![2, 4, 3], vec![56]),
                    (vec![2, 4], vec![8, 8]),
                ] {
                    let platform = platform();
                    let mut storage = new_storage(&platform);
                    for value in 1..=5_i64 {
                        storage.insert(&value).unwrap();
                    }
                    let len = storage.len();

                    for index in &removed {
                        storage.remove((*index).into()).unwrap();
                    }

                    assert_eq!(storage.len(), len, "removed: {removed:?}");
                    assert_eq!(
                        storage.records.free_size(),
                        free.iter().sum::<u64>(),
                        "removed: {removed:?}"
                    );
                    let storage = reopen_killed(&platform);
                    assert_eq!(
                        storage.records.free_size(),
                        free.iter().sum::<u64>(),
                        "removed: {removed:?}"
                    );
                    assert_eq!(storage.value(5.into()), Ok(5_i64), "removed: {removed:?}");
                }
            }

            #[test]
            fn remove_missing_index_should_fail() {
                let platform = platform();
                let mut storage = new_storage(&platform);

                assert_eq!(storage.remove(1.into()), Err(not_found(1)));
            }

            #[test]
            fn remove_failure_should_be_recovered() {
                for removed in [0, 1] {
                    assert_failures_recover(
                        |storage| {
                            vec![
                                storage.insert(&1_i64).unwrap(),
                                storage.insert(&2_i64).unwrap(),
                            ]
                        },
                        |storage, indexes| storage.remove(indexes[removed]),
                    );
                }
            }
        }

        mod rename {
            use super::*;

            #[test]
            fn rename_should_rename_data() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&1_i64).unwrap();

                storage.rename("/renamed").unwrap();

                assert_eq!(storage.name(), "/renamed");
                assert_eq!(storage.value(index), Ok(1_i64));
                assert!(!platform.fs().exists(NAME).unwrap());
            }
        }

        mod replace {
            use super::*;

            #[test]
            fn replace_should_replace_value() {
                for value in [vec![2_i64], vec![2_i64, 3, 4], vec![]] {
                    let platform = platform();
                    let mut storage = new_storage(&platform);
                    let index = storage.insert(&vec![1_i64]).unwrap();
                    let other = storage.insert(&9_i64).unwrap();

                    storage.replace(index, &value).unwrap();

                    assert_eq!(storage.value(index), Ok(value.clone()), "value: {value:?}");
                    assert_eq!(storage.value(other), Ok(9_i64), "value: {value:?}");
                }
            }

            #[test]
            fn replace_missing_index_should_fail() {
                let platform = platform();
                let mut storage = new_storage(&platform);

                assert_eq!(storage.replace(1.into(), &1_i64), Err(not_found(1)));
            }

            #[test]
            fn replace_failure_should_be_recovered() {
                assert_failures_recover(
                    |storage| {
                        vec![
                            storage.insert(&1_i64).unwrap(),
                            storage.insert(&2_i64).unwrap(),
                        ]
                    },
                    |storage, indexes| storage.replace(indexes[0], &vec![3_i64; 3]),
                );
            }

            #[test]
            fn replace_with_smaller_failure_should_be_recovered() {
                assert_failures_recover(
                    |storage| {
                        vec![
                            storage.insert(&vec![1_i64; 5]).unwrap(),
                            storage.insert(&2_i64).unwrap(),
                        ]
                    },
                    |storage, indexes| storage.replace(indexes[0], &3_i64),
                );
            }
        }

        mod resize_value {
            use super::*;

            fn three_values(storage: &mut TestStorage) -> Vec<StorageIndex> {
                vec![
                    storage.insert_bytes(&[1; 8]).unwrap(),
                    storage.insert_bytes(&[2; 8]).unwrap(),
                    storage.insert_bytes(&[3; 8]).unwrap(),
                ]
            }

            #[test]
            fn resize_value_greater_at_end_should_enlarge_storage() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert_bytes(&[1; 8]).unwrap();
                let len = storage.len();

                storage.resize_value(index, 12).unwrap();

                assert_eq!(storage.len(), len + 4);
                assert_eq!(value(&storage, index), [vec![1; 8], vec![0; 4]].concat());
            }

            #[test]
            fn resize_value_greater_should_enlarge_into_following_free_region() {
                for (removed, size, free) in [(8, 32, 0), (8, 16, 0), (40, 16, 32), (40, 30, 18)] {
                    let platform = platform();
                    let mut storage = new_storage(&platform);
                    let index = storage.insert_bytes(&[1; 8]).unwrap();
                    let next = storage.insert_bytes(&vec![2; removed]).unwrap();
                    let other = storage.insert_bytes(&[3]).unwrap();
                    storage.remove(next).unwrap();
                    let len = storage.len();

                    storage.resize_value(index, size).unwrap();

                    let case = format!("removed: {removed}, size: {size}");
                    assert_eq!(storage.len(), len, "{case}");
                    assert_eq!(
                        value(&storage, index),
                        [vec![1; 8], vec![0; size as usize - 8]].concat(),
                        "{case}"
                    );
                    assert_eq!(storage.records.free_size(), free, "{case}");
                    let storage = reopen_killed(&platform);
                    assert_eq!(value(&storage, other), vec![3], "{case}");
                    assert_eq!(storage.records.free_size(), free, "{case}");
                }
            }

            #[test]
            fn resize_value_greater_should_move_value_to_free_region() {
                for (removed, free) in [(16, 8), (40, 8 + 8)] {
                    let platform = platform();
                    let mut storage = new_storage(&platform);
                    let free_index = storage.insert_bytes(&vec![9; removed]).unwrap();
                    let indexes = three_values(&mut storage);
                    storage.remove(free_index).unwrap();
                    let len = storage.len();

                    storage.resize_value(indexes[1], 16).unwrap();

                    assert_eq!(storage.len(), len, "removed: {removed}");
                    assert_eq!(
                        value(&storage, indexes[1]),
                        [vec![2; 8], vec![0; 8]].concat(),
                        "removed: {removed}"
                    );
                    assert_eq!(storage.records.free_size(), free, "removed: {removed}");
                    assert_eq!(
                        value(&storage, indexes[2]),
                        vec![3; 8],
                        "removed: {removed}"
                    );
                }
            }

            #[test]
            fn resize_value_greater_should_move_value_to_end_without_free_region() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let indexes = three_values(&mut storage);
                let len = storage.len();

                storage.resize_value(indexes[1], 16).unwrap();

                assert_eq!(storage.len(), len + 32);
                assert_eq!(
                    value(&storage, indexes[1]),
                    [vec![2; 8], vec![0; 8]].concat()
                );
                assert_eq!(storage.records.free_size(), 8);
            }

            #[test]
            fn resize_value_smaller_at_end_should_truncate_storage() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert_bytes(&[1; 8]).unwrap();
                let len = storage.len();

                storage.resize_value(index, 2).unwrap();

                assert_eq!(storage.len(), len - 6);
                assert_eq!(value(&reopen_killed(&platform), index), vec![1; 2]);
            }

            #[test]
            fn resize_value_smaller_should_free_rest_of_value() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert_bytes(&[1; 40]).unwrap();
                let other = storage.insert_bytes(&[2]).unwrap();
                let len = storage.len();

                storage.resize_value(index, 8).unwrap();

                assert_eq!(storage.len(), len);
                assert_eq!(storage.records.free_size(), 16);
                let storage = reopen_killed(&platform);
                assert_eq!(value(&storage, index), vec![1; 8]);
                assert_eq!(value(&storage, other), vec![2]);
                assert_eq!(storage.records.free_size(), 16);
            }

            #[test]
            fn resize_value_smaller_without_room_for_free_record_should_move_value_to_end() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let indexes = three_values(&mut storage);
                let len = storage.len();

                storage.resize_value(indexes[1], 2).unwrap();

                assert_eq!(storage.len(), len + 18);
                assert_eq!(value(&storage, indexes[1]), vec![2; 2]);
                assert_eq!(storage.records.free_size(), 8);
            }

            #[test]
            fn resize_value_same_should_keep_value() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert_bytes(&[1; 8]).unwrap();
                let before = bytes(&storage);

                storage.resize_value(index, 8).unwrap();

                assert_eq!(bytes(&storage), before);
            }

            #[test]
            fn resize_value_missing_index_should_fail() {
                let platform = platform();
                let mut storage = new_storage(&platform);

                assert_eq!(storage.resize_value(1.into(), 1), Err(not_found(1)));
            }

            #[test]
            fn resize_value_failure_should_be_recovered() {
                for (index, size) in [
                    (2, 16),
                    (0, 16),
                    (1, 16),
                    (1, 2),
                    (2, 2),
                    (0, 30),
                    (3, 16),
                    (3, 64),
                ] {
                    assert_failures_recover(
                        |storage| {
                            let mut indexes = three_values(storage);
                            let free = storage.insert_bytes(&[9; 40]).unwrap();
                            indexes.push(storage.insert_bytes(&[4; 8]).unwrap());
                            storage.remove(free).unwrap();
                            indexes.swap(2, 3);
                            indexes
                        },
                        |storage, indexes| storage.resize_value(indexes[index], size),
                    );
                }
            }

            #[test]
            fn resize_value_shrink_in_middle_failure_should_be_recovered() {
                assert_failures_recover(
                    |storage| {
                        vec![
                            storage.insert_bytes(&[1; 40]).unwrap(),
                            storage.insert_bytes(&[2; 8]).unwrap(),
                        ]
                    },
                    |storage, indexes| storage.resize_value(indexes[0], 2),
                );
            }
        }

        mod transaction {
            use super::*;

            #[test]
            fn transaction_should_return_nested_ids() {
                let platform = platform();
                let mut storage = new_storage(&platform);

                assert_eq!([storage.transaction(), storage.transaction()], [1, 2]);
            }
        }

        mod set_overlay_limit {
            use super::*;

            #[test]
            fn set_overlay_limit_should_set_limit_of_data() {
                let platform = platform();
                let mut storage = new_storage(&platform);

                storage.set_overlay_limit(5);

                assert_eq!(storage.overlay_limit(), 5);
                assert_eq!(storage.data.overlay_limit(), 5);
            }
        }

        mod set_sync_mode {
            use super::*;

            #[test]
            fn set_sync_mode_should_set_mode_of_data() {
                let platform = platform();
                let mut storage = new_storage(&platform);

                storage.set_sync_mode(SyncMode::Commit);

                assert_eq!(storage.sync_mode(), SyncMode::Commit);
                assert_eq!(storage.data.sync_mode(), SyncMode::Commit);
            }
        }

        mod sync {
            use super::*;

            #[test]
            fn sync_should_make_data_survive_crash() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&1_i64).unwrap();

                storage.sync().unwrap();

                platform.fs().crash();
                assert_eq!(new_storage(&platform).value(index), Ok(1_i64));
            }
        }

        mod value {
            use super::*;

            #[test]
            fn value_missing_index_should_fail() {
                let platform = platform();
                let storage = new_storage(&platform);

                assert_eq!(storage.value::<i64>(1.into()), Err(not_found(1)));
            }

            #[test]
            fn value_of_other_type_should_fail() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&1_i64).unwrap();

                assert!(storage.value::<Vec<i64>>(index).is_err());
            }
        }

        mod value_as_bytes_at {
            use super::*;

            #[test]
            fn value_as_bytes_at_should_return_bytes_from_offset() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert_bytes(&[1, 2, 3]).unwrap();

                assert_eq!(
                    storage.value_as_bytes_at(index, 1).unwrap().to_vec(),
                    vec![2, 3]
                );
                assert!(storage.value_as_bytes_at(index, 3).unwrap().is_empty());
            }

            #[test]
            fn value_as_bytes_at_offset_out_of_bounds_should_fail() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert_bytes(&[1, 2, 3]).unwrap();

                assert_eq!(
                    storage.value_as_bytes_at(index, 4),
                    Err(DbError::storage(
                        DbErrorType::OutOfBounds,
                        "Offset (4) out of bounds (3)"
                    ))
                );
            }

            #[test]
            fn value_as_bytes_at_missing_index_should_fail() {
                let platform = platform();
                let storage = new_storage(&platform);

                assert_eq!(storage.value_as_bytes_at(1.into(), 0), Err(not_found(1)));
            }
        }

        mod value_as_bytes_at_size {
            use super::*;

            #[test]
            fn value_as_bytes_at_size_out_of_bounds_should_fail() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert_bytes(&[1, 2, 3]).unwrap();

                assert_eq!(
                    storage.value_as_bytes_at_size(index, 1, 3),
                    Err(DbError::storage(
                        DbErrorType::OutOfBounds,
                        "Value size (4) out of bounds (3)"
                    ))
                );
            }

            #[test]
            fn value_as_bytes_at_size_missing_index_should_fail() {
                let platform = platform();
                let storage = new_storage(&platform);

                assert_eq!(
                    storage.value_as_bytes_at_size(1.into(), 0, 0),
                    Err(not_found(1))
                );
            }
        }

        mod value_at {
            use super::*;

            #[test]
            fn value_at_should_deserialize_value_from_offset() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&vec![1_i64, 2, 3]).unwrap();
                storage.insert_at(index, 0, &2_u64).unwrap();

                assert_eq!(storage.value_at::<i64>(index, 16), Ok(2));
                assert_eq!(storage.value::<Vec<i64>>(index), Ok(vec![1, 2]));
            }

            #[test]
            fn value_at_missing_index_should_fail() {
                let platform = platform();
                let storage = new_storage(&platform);

                assert_eq!(storage.value_at::<i64>(1.into(), 0), Err(not_found(1)));
            }
        }

        mod value_size {
            use super::*;

            #[test]
            fn value_size_should_return_size_of_value() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&"Hello".to_string()).unwrap();

                assert_eq!(storage.value_size(index), Ok(13));
            }

            #[test]
            fn value_size_missing_index_should_fail() {
                let platform = platform();
                let storage = new_storage(&platform);

                assert_eq!(storage.value_size(1.into()), Err(not_found(1)));
            }
        }

        mod set_version {
            use super::*;

            #[test]
            fn set_version_should_overwrite_version_record() {
                let platform = platform();
                let mut storage = new_storage(&platform);
                let index = storage.insert(&1_u64).unwrap();

                storage.set_version(MAX_VERSION).unwrap();

                assert_eq!(storage.version(), MAX_VERSION);
                drop(storage);
                let storage = new_storage(&platform);
                assert_eq!(storage.version(), MAX_VERSION);
                assert_eq!(storage.value(index), Ok(1_u64));
            }

            #[test]
            fn set_version_io_failure_should_keep_version_on_recover() {
                assert_failures_recover(
                    |storage| vec![storage.insert(&1_u64).unwrap()],
                    |storage, _| storage.set_version(MAX_VERSION),
                );
            }
        }
    }
}
