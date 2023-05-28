use super::map_data::MapData;
use super::map_data_memory::MapDataMemory;
use super::map_data_storage_index::MapDataStorageIndex;
use super::map_value_state::MapValueState;
use crate::collections::vec::DbVec;
use crate::collections::vec::VecValue;
use crate::db::db_error::DbError;
use crate::storage::file_storage::FileStorage;
use crate::storage::storage_index::StorageIndex;
use crate::storage::Storage;
use crate::utilities::stable_hash::StableHash;
use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

pub struct MapDataStorage<K, T, Data = FileStorage>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + VecValue,
    T: Clone + Default + Eq + PartialEq + VecValue,
    Data: Storage,
{
    storage: Rc<RefCell<Data>>,
    storage_index: StorageIndex,
    data_index: MapDataStorageIndex,
    states: DbVec<MapValueState, Data>,
    keys: DbVec<K, Data>,
    values: DbVec<T, Data>,
}

impl<K, T, Data> MapDataStorage<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + VecValue,
    T: Clone + Default + Eq + PartialEq + VecValue,
    Data: Storage,
{
    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        let states = DbVec::<MapValueState, Data>::new(storage.clone())?;
        let keys = DbVec::<K, Data>::new(storage.clone())?;
        let values = DbVec::<T, Data>::new(storage.clone())?;

        let data_index = MapDataStorageIndex {
            len: 0,
            states_index: states.storage_index(),
            keys_index: keys.storage_index(),
            values_index: values.storage_index(),
        };

        let storage_index = storage.borrow_mut().insert(&data_index)?;

        Ok(Self {
            storage,
            storage_index,
            data_index,
            states,
            keys,
            values,
        })
    }

    pub fn from_storage(
        storage: Rc<RefCell<Data>>,
        storage_index: StorageIndex,
    ) -> Result<Self, DbError> {
        let data_index = storage
            .borrow_mut()
            .value::<MapDataStorageIndex>(storage_index)?;
        let states =
            DbVec::<MapValueState, Data>::from_storage(storage.clone(), data_index.states_index)?;
        let keys = DbVec::<K, Data>::from_storage(storage.clone(), data_index.keys_index)?;
        let values = DbVec::<T, Data>::from_storage(storage.clone(), data_index.values_index)?;

        Ok(Self {
            storage,
            storage_index,
            data_index,
            states,
            keys,
            values,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_index
    }

    pub fn to_map_data_memory(&self) -> Result<MapDataMemory<K, T>, DbError> {
        Ok(MapDataMemory {
            len: self.data_index.len,
            states: self.states.to_vec()?,
            keys: self.keys.to_vec()?,
            values: self.values.to_vec()?,
        })
    }
}

impl<K, T, Data> MapData<K, T> for MapDataStorage<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + VecValue,
    T: Clone + Default + Eq + PartialEq + VecValue,
    Data: Storage,
{
    fn capacity(&self) -> u64 {
        self.states.len()
    }

    fn commit(&mut self, id: u64) -> Result<(), DbError> {
        self.storage.borrow_mut().commit(id)
    }

    fn len(&self) -> u64 {
        self.data_index.len
    }

    fn key(&self, index: u64) -> Result<K, DbError> {
        self.keys.value(index)
    }

    fn resize(&mut self, capacity: u64) -> Result<(), DbError> {
        self.states.resize(capacity, &MapValueState::Empty)?;
        self.keys.resize(capacity, &K::default())?;
        self.values.resize(capacity, &T::default())
    }

    fn set_len(&mut self, len: u64) -> Result<(), DbError> {
        self.data_index.len = len;
        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, 0, &self.len())
    }

    fn set_state(&mut self, index: u64, state: MapValueState) -> Result<(), DbError> {
        self.states.replace(index, &state)?;
        Ok(())
    }

    fn set_key(&mut self, index: u64, key: &K) -> Result<(), DbError> {
        self.keys.replace(index, key)?;
        Ok(())
    }

    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        self.values.replace(index, value)?;
        Ok(())
    }

    fn state(&self, index: u64) -> Result<MapValueState, DbError> {
        self.states.value(index)
    }

    fn swap(&mut self, index: u64, other: u64) -> Result<(), DbError> {
        self.states.swap(index, other)?;
        self.keys.swap(index, other)?;
        self.values.swap(index, other)
    }

    fn transaction(&mut self) -> u64 {
        self.storage.borrow_mut().transaction()
    }

    fn value(&self, index: u64) -> Result<T, DbError> {
        self.values.value(index)
    }
}
