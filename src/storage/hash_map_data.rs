use crate::DbError;

use super::hash_map_key_value::HashMapKeyValue;
use super::hash_map_meta_value::HashMapMetaValue;
use super::Serialize;
use super::StableHash;
use std::hash::Hash;

pub(crate) trait HashMapData<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
{
    fn capacity(&self) -> u64;
    fn commit(&mut self) -> Result<(), DbError>;
    fn count(&self) -> u64;
    fn meta_value(&self, pos: u64) -> Result<HashMapMetaValue, DbError>;
    fn record(&self, pos: u64) -> Result<HashMapKeyValue<K, T>, DbError>;
    fn set_count(&mut self, new_count: u64) -> Result<(), DbError>;
    fn set_meta_value(&mut self, pos: u64, meta_value: HashMapMetaValue) -> Result<(), DbError>;
    fn set_value(&mut self, pos: u64, value: HashMapKeyValue<K, T>) -> Result<(), DbError>;
    fn set_values(&mut self, values: Vec<HashMapKeyValue<K, T>>) -> Result<(), DbError>;
    fn transaction(&mut self);
    fn values(&mut self) -> Result<Vec<HashMapKeyValue<K, T>>, DbError>;
}
