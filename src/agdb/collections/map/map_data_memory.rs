use super::map_data::MapData;
use super::map_value_state::MapValueState;
use crate::utilities::stable_hash::StableHash;
use crate::DbError;
use std::hash::Hash;

pub struct MapDataMemory<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash,
    T: Clone + Default + Eq + PartialEq,
{
    len: u64,
    states: Vec<MapValueState>,
    keys: Vec<K>,
    values: Vec<T>,
}

impl<K, T> MapDataMemory<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash,
    T: Clone + Default + Eq + PartialEq,
{
    pub fn new() -> Self {
        Self {
            len: 0,
            states: vec![],
            keys: vec![],
            values: vec![],
        }
    }
}

impl<K, T> MapData<K, T> for MapDataMemory<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash,
    T: Clone + Default + Eq + PartialEq,
{
    fn capacity(&self) -> u64 {
        self.states.len() as u64
    }

    fn commit(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    fn len(&self) -> u64 {
        self.len
    }

    fn key(&self, index: u64) -> Result<K, DbError> {
        Ok(self.keys[index as usize].clone())
    }

    fn resize(&mut self, capacity: u64) -> Result<(), DbError> {
        self.states.resize(capacity as usize, MapValueState::Empty);
        self.keys.resize(capacity as usize, K::default());
        self.values.resize(capacity as usize, T::default());

        Ok(())
    }

    fn set_len(&mut self, len: u64) -> Result<(), DbError> {
        self.len = len;

        Ok(())
    }

    fn set_state(&mut self, index: u64, state: MapValueState) -> Result<(), DbError> {
        self.states[index as usize] = state;
        Ok(())
    }

    fn set_key(&mut self, index: u64, key: &K) -> Result<(), DbError> {
        self.keys[index as usize] = key.clone();
        Ok(())
    }

    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        self.values[index as usize] = value.clone();
        Ok(())
    }

    fn state(&self, index: u64) -> Result<MapValueState, DbError> {
        if let Some(s) = self.states.get(index as usize) {
            Ok(s.clone())
        } else {
            Err(DbError::from(format!(
                "MapDataMemory::state() error: index '{}' out of bounds ({})",
                index,
                self.keys.len()
            )))
        }
    }

    fn swap(&mut self, index: u64, other: u64) -> Result<(), DbError> {
        self.states.swap(index as usize, other as usize);
        self.keys.swap(index as usize, other as usize);
        self.values.swap(index as usize, other as usize);
        Ok(())
    }

    fn transaction(&mut self) {}

    fn value(&self, index: u64) -> Result<T, DbError> {
        Ok(self.values[index as usize].clone())
    }
}
