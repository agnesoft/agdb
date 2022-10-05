use super::dictionary_data::DictionaryData;
use super::dictionary_value::DictionaryValue;
use crate::storage::Serialize;
use crate::storage::StableHash;
use crate::DbError;

pub(crate) struct DictionaryImpl<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: DictionaryData<T>,
{
    pub(super) data: Data,
    pub(super) phantom_data: std::marker::PhantomData<T>,
}

impl<T, Data> DictionaryImpl<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: DictionaryData<T>,
{
    pub(crate) fn count(&self, index: i64) -> Result<Option<u64>, DbError> {
        if 0 < index && index < self.data.capacity() as i64 {
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

        for index in self.data.indexes(hash)? {
            let dictionary_value = self.data.value(index)?;

            if dictionary_value.value == *value {
                return Ok(Some(index));
            }
        }

        Ok(None)
    }

    pub(crate) fn insert(&mut self, value: &T) -> Result<i64, DbError> {
        let hash = value.stable_hash();

        for index in self.data.indexes(hash)? {
            let dictionary_value = self.data.value(index)?;

            if dictionary_value.value == *value {
                self.data.set_meta(index, dictionary_value.meta + 1)?;
                return Ok(index);
            }
        }

        let mut free_index = -self.data.meta(0)?;

        self.data.transaction();

        if free_index == 0 {
            free_index = self.data.capacity() as i64;
        } else {
            let next_free_index = self.data.meta(free_index)?;
            self.data.set_meta(0, next_free_index)?;
        }

        self.data.set_value(
            free_index,
            DictionaryValue::<T> {
                meta: 1,
                hash,
                value: value.clone(),
            },
        )?;
        self.data.insert(hash, free_index)?;
        let len = self.len()?;
        self.data.set_hash(0, len + 1)?;
        self.data.commit()?;

        Ok(free_index)
    }

    pub(crate) fn remove(&mut self, index: i64) -> Result<(), DbError> {
        if 0 < index && index < self.data.capacity() as i64 {
            let value = self.data.meta(index)?;

            self.data.transaction();

            if value == 1 {
                let len = self.len()?;
                let hash = self.data.hash(index)?;
                self.data.remove(hash, index)?;
                let next_free_index = self.data.meta(0)?;
                self.data.set_meta(index, next_free_index)?;
                self.data.set_meta(0, -index)?;
                self.data.set_hash(0, len - 1)?;
            } else if 1 < value {
                self.data.set_meta(index, value - 1)?;
            }

            self.data.commit()?;
        }

        Ok(())
    }

    pub(crate) fn value(&mut self, index: i64) -> Result<Option<T>, DbError> {
        if 0 < index && index < self.data.capacity() as i64 {
            let value = self.data.value(index)?;

            if 0 < value.meta {
                return Ok(Some(value.value));
            }
        }

        Ok(None)
    }
}
