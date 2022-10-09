use crate::dictionary_data::DictionaryData;
use crate::dictionary_index::DictionaryIndex;
use crate::dictionary_value::DictionaryValue;
use agdb_db_error::DbError;
use agdb_multi_map::MultiMap;
use agdb_serialize::Serialize;
use agdb_utilities::StableHash;

pub struct DictionaryDataMemory<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    pub(crate) index: MultiMap<u64, DictionaryIndex>,
    pub(crate) values: Vec<DictionaryValue<T>>,
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

    fn indexes(&self, hash: u64) -> Result<Vec<DictionaryIndex>, DbError> {
        self.index.values(&hash)
    }

    fn insert(&mut self, hash: u64, index: &DictionaryIndex) -> Result<(), DbError> {
        self.index.insert(hash, index.clone())
    }

    fn hash(&self, index: &DictionaryIndex) -> Result<u64, DbError> {
        Ok(self.values[index.as_usize()].hash)
    }

    fn meta(&self, index: &DictionaryIndex) -> Result<i64, DbError> {
        Ok(self.values[index.as_usize()].meta)
    }

    fn remove(&mut self, hash: u64, index: &DictionaryIndex) -> Result<(), DbError> {
        self.index.remove_value(&hash, index)?;

        Ok(())
    }

    fn set_hash(&mut self, index: &DictionaryIndex, hash: u64) -> Result<(), DbError> {
        self.values[index.as_usize()].hash = hash;

        Ok(())
    }

    fn set_meta(&mut self, index: &DictionaryIndex, meta: i64) -> Result<(), DbError> {
        self.values[index.as_usize()].meta = meta;

        Ok(())
    }

    fn set_value(
        &mut self,
        index: &DictionaryIndex,
        value: DictionaryValue<T>,
    ) -> Result<(), DbError> {
        if self.capacity() == index.as_u64() {
            self.values.push(value);
        } else {
            self.values[index.as_usize()] = value;
        }

        Ok(())
    }

    fn transaction(&mut self) {}

    fn value(&self, index: &DictionaryIndex) -> Result<DictionaryValue<T>, DbError> {
        Ok(self.values[index.as_usize()].clone())
    }
}
