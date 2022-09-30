use super::hash_map_data::HashMapData;
use super::hash_map_key_value::HashMapKeyValue;
use super::hash_map_meta_value::HashMapMetaValue;
use super::Serialize;
use super::StableHash;
use std::hash::Hash;

pub(crate) struct HashMapDataMemory<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
{
    data: Vec<HashMapKeyValue<K, T>>,
    count: u64,
}

impl<K, T> HashMapData<K, T> for HashMapDataMemory<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
{
    fn capacity(&self) -> u64 {
        self.data.len() as u64
    }

    fn commit(&mut self) -> Result<(), crate::DbError> {
        Ok(())
    }

    fn count(&self) -> u64 {
        self.count
    }

    fn meta_value(&self, pos: u64) -> Result<HashMapMetaValue, crate::DbError> {
        Ok(self.data[pos as usize].meta_value.clone())
    }

    fn record(&self, pos: u64) -> Result<HashMapKeyValue<K, T>, crate::DbError> {
        Ok(self.data[pos as usize].clone())
    }

    fn set_count(&mut self, new_count: u64) -> Result<(), crate::DbError> {
        self.count = new_count;

        Ok(())
    }

    fn set_meta_value(
        &mut self,
        pos: u64,
        meta_value: HashMapMetaValue,
    ) -> Result<(), crate::DbError> {
        self.data[pos as usize].meta_value = meta_value;

        Ok(())
    }

    fn set_value(&mut self, pos: u64, value: HashMapKeyValue<K, T>) -> Result<(), crate::DbError> {
        self.data[pos as usize] = value;

        Ok(())
    }

    fn set_values(&mut self, values: Vec<HashMapKeyValue<K, T>>) -> Result<(), crate::DbError> {
        self.data = values;

        Ok(())
    }

    fn transaction(&mut self) {}

    fn values(&mut self) -> Result<Vec<HashMapKeyValue<K, T>>, crate::DbError> {
        Ok(std::mem::replace(&mut self.data, vec![]))
    }
}
