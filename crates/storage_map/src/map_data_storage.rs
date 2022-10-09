use agdb_db_error::DbError;
use agdb_map_common::MapData;
use agdb_map_common::MapValue;
use agdb_map_common::MapValueState;
use agdb_serialize::Serialize;
use agdb_storage::Storage;
use agdb_storage::StorageIndex;
use agdb_utilities::StableHash;
use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::size_of;
use std::rc::Rc;

pub struct MapDataStorage<K, T, Data: Storage>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
{
    pub(crate) storage: Rc<RefCell<Data>>,
    pub(crate) storage_index: StorageIndex,
    pub(crate) count: u64,
    pub(crate) capacity: u64,
    pub(crate) phantom_data: PhantomData<(K, T)>,
}

impl<K, T, Data> MapDataStorage<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: Storage,
{
    pub fn count(&self) -> u64 {
        self.count
    }

    fn record_offset(pos: u64) -> u64 {
        u64::serialized_size() * 2 + MapValue::<K, T>::serialized_size() * pos
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_index.clone()
    }

    pub fn values(&self) -> Result<Vec<MapValue<K, T>>, DbError> {
        self.storage
            .borrow_mut()
            .value_at::<Vec<MapValue<K, T>>>(&self.storage_index, size_of::<u64>() as u64)
    }
}

impl<K, T, Data> MapData<K, T> for MapDataStorage<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: Storage,
{
    fn capacity(&self) -> u64 {
        self.capacity
    }

    fn commit(&mut self) -> Result<(), DbError> {
        self.storage.borrow_mut().commit()
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn meta_value(&self, pos: u64) -> Result<MapValueState, DbError> {
        self.storage
            .borrow_mut()
            .value_at::<MapValueState>(&self.storage_index, Self::record_offset(pos))
    }

    fn record(&self, pos: u64) -> Result<MapValue<K, T>, DbError> {
        self.storage
            .borrow_mut()
            .value_at::<MapValue<K, T>>(&self.storage_index, Self::record_offset(pos))
    }

    fn set_count(&mut self, new_count: u64) -> Result<(), DbError> {
        self.count = new_count;
        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, 0, &self.count)
    }

    fn set_meta_value(&mut self, pos: u64, meta_value: MapValueState) -> Result<(), DbError> {
        self.storage.borrow_mut().insert_at(
            &self.storage_index,
            Self::record_offset(pos),
            &meta_value,
        )
    }

    fn set_value(&mut self, pos: u64, value: MapValue<K, T>) -> Result<(), DbError> {
        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, Self::record_offset(pos), &value)
    }

    fn set_values(&mut self, values: Vec<MapValue<K, T>>) -> Result<(), DbError> {
        self.capacity = values.len() as u64;
        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, size_of::<u64>() as u64, &values)
    }

    fn take_values(&mut self) -> Result<Vec<MapValue<K, T>>, DbError> {
        self.values()
    }

    fn transaction(&mut self) {
        self.storage.borrow_mut().transaction()
    }
}
