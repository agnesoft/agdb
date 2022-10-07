use super::hash_map_data::HashMapData;
use super::hash_map_key_value::HashMapKeyValue;
use super::hash_map_meta_value::HashMapMetaValue;
use super::StableHash;
use super::Storage;
use super::StorageData;
use crate::DbError;
use serialize::Serialize;
use std::hash::Hash;

pub(crate) struct HashMapDataStorage<K, T, Data: StorageData>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
{
    pub(super) storage: std::rc::Rc<std::cell::RefCell<Storage<Data>>>,
    pub(super) storage_index: i64,
    pub(super) count: u64,
    pub(super) capacity: u64,
    pub(super) phantom_data: std::marker::PhantomData<(K, T)>,
}

impl<K, T, Data> HashMapDataStorage<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: StorageData,
{
    fn record_offset(pos: u64) -> u64 {
        u64::serialized_size() * 2 + HashMapKeyValue::<K, T>::serialized_size() * pos
    }

    pub(super) fn values(&self) -> Result<Vec<HashMapKeyValue<K, T>>, DbError> {
        self.storage
            .borrow_mut()
            .value_at::<Vec<HashMapKeyValue<K, T>>>(
                self.storage_index,
                std::mem::size_of::<u64>() as u64,
            )
    }
}

impl<K, T, Data> HashMapData<K, T> for HashMapDataStorage<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: StorageData,
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

    fn meta_value(&self, pos: u64) -> Result<HashMapMetaValue, DbError> {
        self.storage
            .borrow_mut()
            .value_at::<HashMapMetaValue>(self.storage_index, Self::record_offset(pos))
    }

    fn record(&self, pos: u64) -> Result<HashMapKeyValue<K, T>, DbError> {
        self.storage
            .borrow_mut()
            .value_at::<HashMapKeyValue<K, T>>(self.storage_index, Self::record_offset(pos))
    }

    fn set_count(&mut self, new_count: u64) -> Result<(), DbError> {
        self.count = new_count;
        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, 0, &self.count)
    }

    fn set_meta_value(
        &mut self,
        pos: u64,
        meta_value: HashMapMetaValue,
    ) -> Result<(), crate::DbError> {
        self.storage.borrow_mut().insert_at(
            self.storage_index,
            Self::record_offset(pos),
            &meta_value,
        )
    }

    fn set_value(&mut self, pos: u64, value: HashMapKeyValue<K, T>) -> Result<(), DbError> {
        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, Self::record_offset(pos), &value)
    }

    fn set_values(&mut self, values: Vec<HashMapKeyValue<K, T>>) -> Result<(), DbError> {
        self.capacity = values.len() as u64;
        self.storage.borrow_mut().insert_at(
            self.storage_index,
            std::mem::size_of::<u64>() as u64,
            &values,
        )
    }

    fn take_values(&mut self) -> Result<Vec<HashMapKeyValue<K, T>>, DbError> {
        self.values()
    }

    fn transaction(&mut self) {
        self.storage.borrow_mut().transaction()
    }
}
