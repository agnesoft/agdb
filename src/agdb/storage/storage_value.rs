use super::storage_index::StorageIndex;
use super::Storage;
use crate::utilities::serialize::Serialize;
use crate::DbError;

pub trait StorageValue: Serialize {
    fn store<S: Storage>(&self, storage: &mut S) -> Result<Vec<u8>, DbError>;
    fn load<S: Storage>(storage: &S, bytes: &[u8]) -> Result<Self, DbError>;
    fn remove<S: Storage>(storage: &mut S, bytes: &[u8]) -> Result<(), DbError>;
    fn storage_len() -> u64;
}

impl StorageValue for i64 {
    fn store<S: Storage>(&self, _storage: &mut S) -> Result<Vec<u8>, DbError> {
        Ok(self.serialize())
    }

    fn load<S: Storage>(_storage: &S, bytes: &[u8]) -> Result<Self, DbError> {
        i64::deserialize(bytes)
    }

    fn remove<S: Storage>(_storage: &mut S, _bytes: &[u8]) -> Result<(), DbError> {
        Ok(())
    }

    fn storage_len() -> u64 {
        i64::static_serialized_size()
    }
}

impl StorageValue for u64 {
    fn store<S: Storage>(&self, _storage: &mut S) -> Result<Vec<u8>, DbError> {
        Ok(self.serialize())
    }

    fn load<S: Storage>(_storage: &S, bytes: &[u8]) -> Result<Self, DbError> {
        u64::deserialize(bytes)
    }

    fn remove<S: Storage>(_storage: &mut S, _bytes: &[u8]) -> Result<(), DbError> {
        Ok(())
    }

    fn storage_len() -> u64 {
        u64::static_serialized_size()
    }
}

impl StorageValue for String {
    fn store<S: Storage>(&self, storage: &mut S) -> Result<Vec<u8>, DbError> {
        let index = storage.insert(self)?;
        Ok(index.serialize())
    }

    fn load<S: Storage>(storage: &S, bytes: &[u8]) -> Result<Self, DbError> {
        let index = StorageIndex::deserialize(bytes)?;
        storage.value(&index)
    }

    fn remove<S: Storage>(storage: &mut S, bytes: &[u8]) -> Result<(), DbError> {
        let index = StorageIndex::deserialize(bytes)?;
        storage.remove(&index)
    }

    fn storage_len() -> u64 {
        StorageIndex::static_serialized_size()
    }
}
