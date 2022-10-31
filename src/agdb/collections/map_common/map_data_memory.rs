use super::map_data::MapData;
use super::map_value::MapValue;
use super::map_value_state::MapValueState;
use crate::db_error::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::stable_hash::StableHash;
use std::hash::Hash;
use std::mem::take;

pub struct MapDataMemory<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
{
    pub(crate) data: Vec<MapValue<K, T>>,
    pub(crate) count: u64,
}

impl<K, T> MapData<K, T> for MapDataMemory<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
{
    fn capacity(&self) -> u64 {
        self.data.len() as u64
    }

    fn commit(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn meta_value(&self, pos: u64) -> Result<MapValueState, DbError> {
        Ok(self.data[pos as usize].state.clone())
    }

    fn record(&self, pos: u64) -> Result<MapValue<K, T>, DbError> {
        Ok(self.data[pos as usize].clone())
    }

    fn set_count(&mut self, new_count: u64) -> Result<(), DbError> {
        self.count = new_count;

        Ok(())
    }

    fn set_meta_value(&mut self, pos: u64, meta_value: MapValueState) -> Result<(), DbError> {
        self.data[pos as usize].state = meta_value;

        Ok(())
    }

    fn set_value(&mut self, pos: u64, value: MapValue<K, T>) -> Result<(), DbError> {
        self.data[pos as usize] = value;

        Ok(())
    }

    fn set_values(&mut self, values: Vec<MapValue<K, T>>) -> Result<(), DbError> {
        self.data = values;

        Ok(())
    }

    fn take_values(&mut self) -> Result<Vec<MapValue<K, T>>, DbError> {
        Ok(take(&mut self.data))
    }

    fn transaction(&mut self) {}
}

impl<K, T> Default for MapDataMemory<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
{
    fn default() -> Self {
        Self {
            data: vec![MapValue::<K, T>::default()],
            count: 0,
        }
    }
}
