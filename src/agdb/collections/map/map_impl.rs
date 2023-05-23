use super::multi_map_impl::MultiMapImpl;
use crate::collections::map::map_data::MapData;
use crate::collections::map::map_iterator::MapIterator;
use crate::db::db_error::DbError;
use crate::utilities::stable_hash::StableHash;
use std::hash::Hash;

pub struct MapImpl<K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + PartialEq,
    Data: MapData<K, T>,
{
    pub(crate) multi_map: MultiMapImpl<K, T, Data>,
}

impl<K, T, Data> MapImpl<K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + PartialEq,
    Data: MapData<K, T>,
{
    #[allow(dead_code)]
    pub fn capacity(&self) -> u64 {
        self.multi_map.capacity()
    }

    #[allow(dead_code)]
    pub fn contains(&self, key: &K) -> Result<bool, DbError> {
        self.multi_map.contains(key)
    }

    #[allow(dead_code)]
    pub fn contains_value(&self, key: &K, value: &T) -> Result<bool, DbError> {
        self.multi_map.contains_value(key, value)
    }

    pub fn insert(&mut self, key: &K, value: &T) -> Result<Option<T>, DbError> {
        let old_value = self.value(key)?;

        if let Some(old) = &old_value {
            self.multi_map.replace(key, old, value)?;
        } else {
            self.multi_map.insert(key, value)?;
        }

        Ok(old_value)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.multi_map.is_empty()
    }

    pub fn iter(&self) -> MapIterator<K, T, Data> {
        self.multi_map.iter()
    }

    pub fn len(&self) -> u64 {
        self.multi_map.len()
    }

    pub fn remove(&mut self, key: &K) -> Result<(), DbError> {
        self.multi_map.remove_key(key)
    }

    #[allow(dead_code)]
    pub fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        self.multi_map.reserve(capacity)
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        self.multi_map.value(key)
    }
}
