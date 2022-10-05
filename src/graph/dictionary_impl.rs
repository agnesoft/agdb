use super::dictionary_data::DictionaryData;
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
    pub(crate) fn count(&self) -> u64 {
        todo!()
    }

    pub(crate) fn len(&self) -> u64 {
        todo!()
    }

    pub(crate) fn index(&self, value: &T) -> Result<Option<i64>, DbError> {
        todo!()
    }

    pub(crate) fn insert(&mut self, index: i64, value: &T) -> Result<(), DbError> {
        todo!()
    }

    pub(crate) fn remove(&mut self, index: i64) -> Result<(), DbError> {
        todo!()
    }

    pub(crate) fn value(&mut self, index: i64) -> Result<Option<T>, DbError> {
        todo!()
    }
}
