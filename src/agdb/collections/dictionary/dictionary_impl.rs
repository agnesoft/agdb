use super::dictionary_data::DictionaryData;
use super::dictionary_index::DictionaryIndex;
use super::dictionary_value::DictionaryValue;
use crate::db_error::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::stable_hash::StableHash;
use std::marker::PhantomData;

pub struct DictionaryImpl<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: DictionaryData<T>,
{
    pub(crate) data: Data,
    pub(crate) phantom_data: PhantomData<T>,
}

#[allow(dead_code)]
impl<T, Data> DictionaryImpl<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: DictionaryData<T>,
{
    pub fn count(&self, index: &DictionaryIndex) -> Result<Option<u64>, DbError> {
        let mut c = None;

        if self.is_valid_index(index) {
            let value = self.data.meta(index)?;

            if 0 < value {
                c = Some(value as u64);
            }
        }

        Ok(c)
    }

    pub fn len(&self) -> Result<u64, DbError> {
        self.data.hash(&DictionaryIndex::default())
    }

    pub fn index(&self, value: &T) -> Result<Option<DictionaryIndex>, DbError> {
        let hash = value.stable_hash();

        if let Some(value) = self.find_value(hash, value)? {
            return Ok(Some(value.0));
        }

        Ok(None)
    }

    pub fn insert(&mut self, value: &T) -> Result<DictionaryIndex, DbError> {
        let hash = value.stable_hash();
        let index;

        self.data.transaction();

        if let Some(value) = self.find_value(hash, value)? {
            index = value.0;
            self.data.set_meta(&index, value.1 + 1)?;
        } else {
            index = self.insert_new(hash, value)?;
        }

        self.data.commit()?;

        Ok(index)
    }

    pub fn remove(&mut self, index: &DictionaryIndex) -> Result<(), DbError> {
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

    pub fn value(&self, index: &DictionaryIndex) -> Result<Option<T>, DbError> {
        let mut v = None;

        if self.is_valid_index(index) {
            let value = self.data.value(index)?;

            if 0 < value.meta {
                v = Some(value.value);
            }
        }

        Ok(v)
    }

    fn find_value(&self, hash: u64, value: &T) -> Result<Option<(DictionaryIndex, i64)>, DbError> {
        for index in self.data.indexes(hash)? {
            let dictionary_value = self.data.value(&index)?;

            if dictionary_value.value == *value {
                return Ok(Some((index, dictionary_value.meta)));
            }
        }

        Ok(None)
    }

    fn free_index(&mut self, index: &DictionaryIndex) -> Result<(), DbError> {
        let next_free_index = self.data.meta(&DictionaryIndex::default())?;
        self.data.set_meta(index, next_free_index)?;
        self.data
            .set_meta(&DictionaryIndex::default(), -index.value())
    }

    fn get_free_index(&mut self) -> Result<DictionaryIndex, DbError> {
        let mut free_index = -self.data.meta(&DictionaryIndex::default())?;

        if free_index == 0 {
            free_index = self.data.capacity() as i64;
        } else {
            let next_free_index = self.data.meta(&DictionaryIndex::from(free_index))?;
            self.data
                .set_meta(&DictionaryIndex::default(), next_free_index)?;
        }

        Ok(DictionaryIndex::from(free_index))
    }

    fn insert_new(&mut self, hash: u64, value: &T) -> Result<DictionaryIndex, DbError> {
        let index = self.get_free_index()?;

        self.data.insert(hash, &index)?;
        self.data.set_value(
            &index,
            DictionaryValue::<T> {
                meta: 1,
                hash,
                value: value.clone(),
            },
        )?;

        let len = self.len()?;
        self.data.set_hash(&DictionaryIndex::default(), len + 1)?;

        Ok(index)
    }

    fn is_valid_index(&self, index: &DictionaryIndex) -> bool {
        index.is_valid() && index.value() < self.data.capacity() as i64
    }

    fn remove_value(&mut self, index: &DictionaryIndex) -> Result<(), DbError> {
        let hash = self.data.hash(index)?;
        self.data.remove(hash, index)?;

        self.free_index(index)?;

        let len = self.len()?;
        self.data.set_hash(&DictionaryIndex::default(), len - 1)?;

        Ok(())
    }
}
