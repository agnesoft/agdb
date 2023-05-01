use super::map_value_state::MapValueState;
use crate::db::db_error::DbError;
use crate::utilities::stable_hash::StableHash;
use std::hash::Hash;

pub trait MapData<K, T>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + PartialEq,
{
    fn capacity(&self) -> u64;
    fn commit(&mut self) -> Result<(), DbError>;
    fn len(&self) -> u64;
    fn key(&self, index: u64) -> Result<K, DbError>;
    fn resize(&mut self, capacity: u64) -> Result<(), DbError>;
    fn set_len(&mut self, len: u64) -> Result<(), DbError>;
    fn set_state(&mut self, index: u64, state: MapValueState) -> Result<(), DbError>;
    fn set_key(&mut self, index: u64, key: &K) -> Result<(), DbError>;
    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError>;
    fn state(&self, index: u64) -> Result<MapValueState, DbError>;
    fn swap(&mut self, index: u64, other: u64) -> Result<(), DbError>;
    fn transaction(&mut self);
    fn value(&self, index: u64) -> Result<T, DbError>;
}
