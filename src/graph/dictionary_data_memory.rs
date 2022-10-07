use super::dictionary_data::DictionaryData;
use super::dictionary_value::DictionaryValue;
use crate::storage::HashMultiMap;
use crate::storage::StableHash;
use crate::DbError;
use serialize::Serialize;

pub(crate) struct DictionaryDataMemory<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    pub(super) index: HashMultiMap<u64, i64>,
    pub(super) values: Vec<DictionaryValue<T>>,
}

impl<T> DictionaryData<T> for DictionaryDataMemory<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    fn capacity(&self) -> u64 {
        self.values.len() as u64
    }

    fn commit(&mut self) -> Result<(), DbError> {
        Ok(())
    }

    fn indexes(&self, hash: u64) -> Result<Vec<i64>, DbError> {
        self.index.values(&hash)
    }

    fn insert(&mut self, hash: u64, index: i64) -> Result<(), DbError> {
        self.index.insert(hash, index)
    }

    fn hash(&self, index: i64) -> Result<u64, DbError> {
        Ok(self.values[index as usize].hash)
    }

    fn meta(&self, index: i64) -> Result<i64, DbError> {
        Ok(self.values[index as usize].meta)
    }

    fn remove(&mut self, hash: u64, index: i64) -> Result<(), DbError> {
        self.index.remove_value(&hash, &index)?;

        Ok(())
    }

    fn set_hash(&mut self, index: i64, hash: u64) -> Result<(), DbError> {
        self.values[index as usize].hash = hash;

        Ok(())
    }

    fn set_meta(&mut self, index: i64, meta: i64) -> Result<(), DbError> {
        self.values[index as usize].meta = meta;

        Ok(())
    }

    fn set_value(&mut self, index: i64, value: DictionaryValue<T>) -> Result<(), DbError> {
        if self.capacity() == index as u64 {
            self.values.push(value);
        } else {
            self.values[index as usize] = value;
        }

        Ok(())
    }

    fn transaction(&mut self) {}

    fn value(&self, index: i64) -> Result<DictionaryValue<T>, crate::DbError> {
        Ok(self.values[index as usize].clone())
    }
}
