use super::dictionary_data::DictionaryData;
use crate::collections::multi_map::MultiMap;
use crate::db::db_error::DbError;
use crate::storage::storage_value::StorageValue;
use crate::utilities::stable_hash::StableHash;

pub struct DictionaryDataMemory<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + StorageValue,
{
    pub(super) index: MultiMap<u64, u64>,
    pub(super) counts: Vec<u64>,
    pub(super) hashes: Vec<u64>,
    pub(super) values: Vec<T>,
}

impl<T> DictionaryDataMemory<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + StorageValue,
{
    pub fn new() -> Self {
        Self {
            index: MultiMap::new(),
            counts: vec![0],
            hashes: vec![0],
            values: vec![T::default()],
        }
    }
}

impl<T> DictionaryData<T> for DictionaryDataMemory<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + StorageValue,
{
    fn capacity(&self) -> u64 {
        self.counts.len() as u64
    }

    fn commit(&mut self, _id: u64) -> Result<(), DbError> {
        Ok(())
    }

    fn indexes(&self, hash: u64) -> Result<Vec<u64>, DbError> {
        self.index.values(&hash)
    }

    fn insert(&mut self, hash: u64, index: u64) -> Result<(), DbError> {
        self.index.insert(&hash, &index)
    }

    fn hash(&self, index: u64) -> Result<u64, DbError> {
        Ok(self.hashes[index as usize])
    }

    fn count(&self, index: u64) -> Result<u64, DbError> {
        Ok(self.counts[index as usize])
    }

    fn remove(&mut self, hash: u64, index: u64) -> Result<(), DbError> {
        self.index.remove_value(&hash, &index)?;

        Ok(())
    }

    fn set_capacity(&mut self, capacity: u64) -> Result<(), DbError> {
        self.counts.resize(capacity as usize, 0);
        self.hashes.resize(capacity as usize, 0);
        self.values.resize(capacity as usize, T::default());

        Ok(())
    }

    fn set_count(&mut self, index: u64, count: u64) -> Result<(), DbError> {
        self.counts[index as usize] = count;

        Ok(())
    }

    fn set_hash(&mut self, index: u64, hash: u64) -> Result<(), DbError> {
        self.hashes[index as usize] = hash;

        Ok(())
    }

    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        self.values[index as usize] = value.clone();

        Ok(())
    }

    fn transaction(&mut self) -> u64 {
        0
    }

    fn value(&self, index: u64) -> Result<T, DbError> {
        Ok(self.values[index as usize].clone())
    }
}
