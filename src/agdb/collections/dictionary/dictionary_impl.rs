use super::dictionary_data::DictionaryData;
use super::dictionary_index::DictionaryIndex;
use crate::db::db_error::DbError;
use crate::storage::storage_value::StorageValue;
use crate::utilities::stable_hash::StableHash;
use std::marker::PhantomData;

pub struct DictionaryImpl<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + StorageValue,
    Data: DictionaryData<T>,
{
    pub data: Data,
    pub phantom_data: PhantomData<T>,
}

#[allow(dead_code)]
impl<T, Data> DictionaryImpl<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + StorageValue,
    Data: DictionaryData<T>,
{
    pub fn count(&self, index: DictionaryIndex) -> Result<Option<u64>, DbError> {
        if self.is_valid_index(index.0) {
            let count = self.data.count(index.0)?;

            if count != 0 {
                return Ok(Some(count));
            }
        }

        Ok(None)
    }

    pub fn len(&self) -> Result<u64, DbError> {
        self.data.count(0)
    }

    pub fn index(&self, value: &T) -> Result<Option<DictionaryIndex>, DbError> {
        self.find_value(value.stable_hash(), value)
    }

    pub fn insert(&mut self, value: &T) -> Result<DictionaryIndex, DbError> {
        let hash = value.stable_hash();
        let index;

        self.data.transaction();

        if let Some(i) = self.find_value(hash, value)? {
            index = i;
            let count = self.data.count(index.0)?;
            self.data.set_count(index.0, count + 1)?;
        } else {
            index = DictionaryIndex(self.insert_new(hash, value)?);
        }

        self.data.commit()?;

        Ok(index)
    }

    pub fn remove(&mut self, index: DictionaryIndex) -> Result<(), DbError> {
        if self.is_valid_index(index.0) {
            let count = self.data.count(index.0)?;

            if count != 0 {
                self.data.transaction();

                if count == 1 {
                    self.remove_value(index.0)?
                } else {
                    self.data.set_count(index.0, count - 1)?
                }

                self.data.commit()?;
            }
        }

        Ok(())
    }

    pub fn value(&self, index: DictionaryIndex) -> Result<Option<T>, DbError> {
        let mut v = None;

        if self.is_valid_index(index.0) && self.data.count(index.0)? != 0 {
            v = Some(self.data.value(index.0)?);
        }

        Ok(v)
    }

    fn find_value(&self, hash: u64, value: &T) -> Result<Option<DictionaryIndex>, DbError> {
        for index in self.data.indexes(hash)? {
            if *value == self.data.value(index)? {
                return Ok(Some(DictionaryIndex(index)));
            }
        }

        Ok(None)
    }

    fn free_index(&mut self, index: u64) -> Result<(), DbError> {
        let next_free_index = self.data.hash(0)?;
        self.data.set_hash(index, next_free_index)?;
        self.data.set_hash(0, index)
    }

    fn get_free_index(&mut self) -> Result<u64, DbError> {
        let mut free_index = self.data.hash(0)?;

        if free_index == 0 {
            free_index = self.data.capacity();
            self.data.set_capacity(free_index + 1)?;
        } else {
            let next_free_index = self.data.hash(free_index)?;
            self.data.set_hash(0, next_free_index)?;
        }

        Ok(free_index)
    }

    fn insert_new(&mut self, hash: u64, value: &T) -> Result<u64, DbError> {
        let index = self.get_free_index()?;

        self.data.insert(hash, index)?;
        self.data.set_hash(index, hash)?;
        self.data.set_count(index, 1)?;
        self.data.set_value(index, value)?;

        let len = self.len()?;
        self.data.set_count(0, len + 1)?;

        Ok(index)
    }

    fn is_valid_index(&self, index: u64) -> bool {
        index != 0 && index < self.data.capacity()
    }

    fn remove_value(&mut self, index: u64) -> Result<(), DbError> {
        let hash = self.data.hash(index)?;
        self.data.remove(hash, index)?;
        self.free_index(index)?;
        self.data.set_count(index, 0)?;
        let len = self.len()?;
        self.data.set_count(0, len - 1)
    }
}
