use super::map_value::MapValue;
use super::map_value_state::MapValueState;
use crate::db::db_error::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::stable_hash::StableHash;
use std::hash::Hash;

pub trait MapData<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
{
    fn capacity(&self) -> u64;
    fn commit(&mut self) -> Result<(), DbError>;
    fn count(&self) -> u64;
    fn meta_value(&self, pos: u64) -> Result<MapValueState, DbError>;
    fn record(&self, pos: u64) -> Result<MapValue<K, T>, DbError>;
    fn set_count(&mut self, new_count: u64) -> Result<(), DbError>;
    fn set_meta_value(&mut self, pos: u64, meta_value: MapValueState) -> Result<(), DbError>;
    fn set_value(&mut self, pos: u64, value: MapValue<K, T>) -> Result<(), DbError>;
    fn set_values(&mut self, values: Vec<MapValue<K, T>>) -> Result<(), DbError>;
    fn take_values(&mut self) -> Result<Vec<MapValue<K, T>>, DbError>;
    fn transaction(&mut self);
}
