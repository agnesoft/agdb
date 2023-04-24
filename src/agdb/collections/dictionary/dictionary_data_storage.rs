use super::dictionary_data::DictionaryData;
use super::dictionary_data_memory::DictionaryDataMemory;
use crate::collections::multi_map_storage::MultiMapStorage;
use crate::collections::vec_storage::VecStorage;
use crate::db::db_error::DbError;
use crate::storage::file_storage::FileStorage;
use crate::storage::storage_index::StorageIndex;
use crate::storage::storage_value::StorageValue;
use crate::storage::Storage;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize_static::SerializeStatic;
use crate::utilities::stable_hash::StableHash;
use std::cell::RefCell;
use std::rc::Rc;

pub struct DictionaryDataStorage<T, Data = FileStorage>
where
    T: Clone + Default + Eq + PartialEq + StableHash + StorageValue,
    Data: Storage,
{
    storage: Rc<RefCell<Data>>,
    storage_index: StorageIndex,
    index: MultiMapStorage<u64, u64, Data>,
    counts: VecStorage<u64, Data>,
    hashes: VecStorage<u64, Data>,
    values: VecStorage<T, Data>,
}

struct DictionaryDataStorageIndexes {
    index_index: StorageIndex,
    counts_index: StorageIndex,
    hashes_index: StorageIndex,
    values_index: StorageIndex,
}

impl<T, Data> DictionaryDataStorage<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + StorageValue,
    Data: Storage,
{
    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        let index = MultiMapStorage::<u64, u64, Data>::new(storage.clone())?;
        let mut counts = VecStorage::<u64, Data>::new(storage.clone())?;
        let mut hashes = VecStorage::<u64, Data>::new(storage.clone())?;
        let mut values = VecStorage::<T, Data>::new(storage.clone())?;

        storage.borrow_mut().transaction();

        counts.push(&0)?;
        hashes.push(&0)?;
        values.push(&T::default())?;

        let data_index = DictionaryDataStorageIndexes {
            index_index: index.storage_index(),
            counts_index: counts.storage_index(),
            hashes_index: hashes.storage_index(),
            values_index: values.storage_index(),
        };

        let storage_index = storage.borrow_mut().insert(&data_index)?;

        storage.borrow_mut().commit()?;

        Ok(Self {
            storage,
            storage_index,
            index,
            counts,
            hashes,
            values,
        })
    }

    pub fn from_storage(
        storage: Rc<RefCell<Data>>,
        storage_index: &StorageIndex,
    ) -> Result<Self, DbError> {
        let data_index = storage
            .borrow_mut()
            .value::<DictionaryDataStorageIndexes>(storage_index)?;
        let index = MultiMapStorage::<u64, u64, Data>::from_storage(
            storage.clone(),
            &data_index.index_index,
        )?;
        let counts =
            VecStorage::<u64, Data>::from_storage(storage.clone(), &data_index.counts_index)?;
        let hashes =
            VecStorage::<u64, Data>::from_storage(storage.clone(), &data_index.hashes_index)?;
        let values =
            VecStorage::<T, Data>::from_storage(storage.clone(), &data_index.values_index)?;

        Ok(Self {
            storage,
            storage_index: *storage_index,
            index,
            counts,
            hashes,
            values,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_index
    }

    pub fn to_dictionary_data_memory(&self) -> Result<DictionaryDataMemory<T>, DbError> {
        Ok(DictionaryDataMemory {
            index: self.index.to_multi_map()?,
            counts: self.counts.to_vec()?,
            hashes: self.hashes.to_vec()?,
            values: self.values.to_vec()?,
        })
    }
}

impl<T, Data> DictionaryData<T> for DictionaryDataStorage<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + StorageValue,
    Data: Storage,
{
    fn capacity(&self) -> u64 {
        self.counts.len()
    }

    fn commit(&mut self) -> Result<(), DbError> {
        self.storage.borrow_mut().commit()
    }

    fn count(&self, index: u64) -> Result<u64, DbError> {
        self.counts.value(index)
    }

    fn indexes(&self, hash: u64) -> Result<Vec<u64>, DbError> {
        self.index.values(&hash)
    }

    fn insert(&mut self, hash: u64, index: u64) -> Result<(), DbError> {
        self.index.insert(&hash, &index)
    }

    fn hash(&self, index: u64) -> Result<u64, DbError> {
        self.hashes.value(index)
    }

    fn remove(&mut self, hash: u64, index: u64) -> Result<(), DbError> {
        self.index.remove_value(&hash, &index)
    }

    fn set_capacity(&mut self, capacity: u64) -> Result<(), DbError> {
        self.counts.resize(capacity, &0)?;
        self.hashes.resize(capacity, &0)?;
        self.values.resize(capacity, &T::default())
    }

    fn set_count(&mut self, index: u64, count: u64) -> Result<(), DbError> {
        self.counts.set_value(index, &count)
    }

    fn set_hash(&mut self, index: u64, hash: u64) -> Result<(), DbError> {
        self.hashes.set_value(index, &hash)
    }

    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        self.values.set_value(index, value)
    }

    fn transaction(&mut self) {
        self.storage.borrow_mut().transaction()
    }

    fn value(&self, index: u64) -> Result<T, DbError> {
        self.values.value(index)
    }
}

impl Serialize for DictionaryDataStorageIndexes {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(self.serialized_size() as usize);
        bytes.extend(self.index_index.serialize());
        bytes.extend(self.counts_index.serialize());
        bytes.extend(self.hashes_index.serialize());
        bytes.extend(self.values_index.serialize());

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        if bytes.len() < Self::static_serialized_size() as usize {
            return Err(DbError::from(
                "DictionaryDataStorageIndexes deserialization error: not enough data",
            ));
        }

        Ok(DictionaryDataStorageIndexes {
            index_index: StorageIndex::deserialize(bytes)?,
            counts_index: StorageIndex::deserialize(
                &bytes[StorageIndex::static_serialized_size() as usize..],
            )?,
            hashes_index: StorageIndex::deserialize(
                &bytes[(StorageIndex::static_serialized_size() * 2) as usize..],
            )?,
            values_index: StorageIndex::deserialize(
                &bytes[(StorageIndex::static_serialized_size() * 3) as usize..],
            )?,
        })
    }

    fn serialized_size(&self) -> u64 {
        Self::static_serialized_size()
    }
}

impl SerializeStatic for DictionaryDataStorageIndexes {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_deserialize() {
        assert_eq!(
            DictionaryDataStorageIndexes::deserialize(&Vec::<u8>::new())
                .err()
                .unwrap(),
            DbError::from("DictionaryDataStorageIndexes deserialization error: not enough data")
        );
    }
}
