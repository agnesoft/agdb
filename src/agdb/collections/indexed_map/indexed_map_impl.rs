use crate::collections::map::map_data::MapData;
use crate::collections::map::map_impl::MapImpl;
use crate::collections::map::map_iterator::MapIterator;
use crate::utilities::stable_hash::StableHash;
use crate::DbError;
use std::hash::Hash;

pub struct IndexedMapImpl<K, T, DataKT, DataTK>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + Hash + PartialEq + StableHash,
    DataKT: MapData<K, T>,
    DataTK: MapData<T, K>,
{
    pub(crate) keys_to_values: MapImpl<K, T, DataKT>,
    pub(crate) values_to_keys: MapImpl<T, K, DataTK>,
}

#[allow(dead_code)]
impl<K, T, DataKT, DataTK> IndexedMapImpl<K, T, DataKT, DataTK>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + Hash + PartialEq + StableHash,
    DataKT: MapData<K, T>,
    DataTK: MapData<T, K>,
{
    pub fn insert(&mut self, key: &K, value: &T) -> Result<(), DbError> {
        if let Some(v) = self.keys_to_values.insert(key, value)? {
            self.values_to_keys.remove(&v)?;
        }

        if let Some(k) = self.values_to_keys.insert(value, key)? {
            self.keys_to_values.remove(&k)?;
        }

        Ok(())
    }

    pub fn iter(&self) -> MapIterator<K, T, DataKT> {
        self.keys_to_values.iter()
    }

    pub fn key(&self, value: &T) -> Result<Option<K>, DbError> {
        self.values_to_keys.value(value)
    }

    pub fn remove_key(&mut self, key: &K) -> Result<(), DbError> {
        if let Some(value) = self.keys_to_values.value(key)? {
            self.values_to_keys.remove(&value)?;
        }

        self.keys_to_values.remove(key)
    }

    pub fn remove_value(&mut self, value: &T) -> Result<(), DbError> {
        if let Some(key) = self.values_to_keys.value(value)? {
            self.keys_to_values.remove(&key)?;
        }

        self.values_to_keys.remove(value)
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        self.keys_to_values.value(key)
    }
}
