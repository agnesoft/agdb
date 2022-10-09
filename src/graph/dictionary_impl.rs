use super::dictionary_data::DictionaryData;
use super::dictionary_value::DictionaryValue;
use crate::storage::StableHash;
use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use std::marker::PhantomData;

pub(crate) struct DictionaryImpl<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: DictionaryData<T>,
{
    pub(super) data: Data,
    pub(super) phantom_data: PhantomData<T>,
}

#[allow(dead_code)]
impl<T, Data> DictionaryImpl<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: DictionaryData<T>,
{
    pub(crate) fn count(&self, index: i64) -> Result<Option<u64>, DbError> {
        if self.is_valid_index(index) {
            let value = self.data.meta(index)?;

            if 0 < value {
                return Ok(Some(value as u64));
            }
        }

        Ok(None)
    }

    pub(crate) fn len(&self) -> Result<u64, DbError> {
        self.data.hash(0)
    }

    pub(crate) fn index(&self, value: &T) -> Result<Option<i64>, DbError> {
        let hash = value.stable_hash();

        if let Some(value) = self.find_value(hash, value)? {
            return Ok(Some(value.0));
        }

        Ok(None)
    }

    pub(crate) fn insert(&mut self, value: &T) -> Result<i64, DbError> {
        let hash = value.stable_hash();
        let index;

        self.data.transaction();

        if let Some(value) = self.find_value(hash, value)? {
            index = value.0;
            self.data.set_meta(index, value.1 + 1)?;
        } else {
            index = self.insert_new(hash, value)?;
        }

        self.data.commit()?;

        Ok(index)
    }

    pub(crate) fn remove(&mut self, index: i64) -> Result<(), DbError> {
        if self.is_valid_index(index) {
            let value = self.data.meta(index)?;

            self.data.transaction();

            if value == 1 {
                self.remove_value(index)?
            } else {
                self.data.set_meta(index, value - 1)?
            }

            self.data.commit()?;
        }

        Ok(())
    }

    pub(crate) fn value(&self, index: i64) -> Result<Option<T>, DbError> {
        if self.is_valid_index(index) {
            let value = self.data.value(index)?;

            if 0 < value.meta {
                return Ok(Some(value.value));
            }
        }

        Ok(None)
    }

    fn find_value(&self, hash: u64, value: &T) -> Result<Option<(i64, i64)>, DbError> {
        for index in self.data.indexes(hash)? {
            let dictionary_value = self.data.value(index)?;

            if dictionary_value.value == *value {
                return Ok(Some((index, dictionary_value.meta)));
            }
        }

        Ok(None)
    }

    fn free_index(&mut self, index: i64) -> Result<(), DbError> {
        let next_free_index = self.data.meta(0)?;
        self.data.set_meta(index, next_free_index)?;
        self.data.set_meta(0, -index)
    }

    fn get_free_index(&mut self) -> Result<i64, DbError> {
        let mut free_index = -self.data.meta(0)?;

        if free_index == 0 {
            free_index = self.data.capacity() as i64;
        } else {
            let next_free_index = self.data.meta(free_index)?;
            self.data.set_meta(0, next_free_index)?;
        }

        Ok(free_index)
    }

    fn insert_new(&mut self, hash: u64, value: &T) -> Result<i64, DbError> {
        let index = self.get_free_index()?;

        self.data.insert(hash, index)?;
        self.data.set_value(
            index,
            DictionaryValue::<T> {
                meta: 1,
                hash,
                value: value.clone(),
            },
        )?;

        let len = self.len()?;
        self.data.set_hash(0, len + 1)?;

        Ok(index)
    }

    fn is_valid_index(&self, index: i64) -> bool {
        0 < index && index < self.data.capacity() as i64
    }

    fn remove_value(&mut self, index: i64) -> Result<(), DbError> {
        let hash = self.data.hash(index)?;
        self.data.remove(hash, index)?;

        self.free_index(index)?;

        let len = self.len()?;
        self.data.set_hash(0, len - 1)?;

        Ok(())
    }
}
