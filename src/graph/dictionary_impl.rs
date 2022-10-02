use super::dictionary_data::DictionaryData;
use crate::DbError;

pub(crate) struct DictionaryImpl<Data: DictionaryData> {
    pub(super) data: Data,
}

impl<Data: DictionaryData> DictionaryImpl<Data> {
    pub(crate) fn index<T>(&self, value: &T) -> Result<Option<i64>, DbError> {
        todo!()
    }

    pub(crate) fn insert<T>(&mut self, index: i64, value: &T) -> Result<(), DbError> {
        todo!()
    }

    pub(crate) fn remove(&mut self, index: i64) -> Result<(), DbError> {
        todo!()
    }

    pub(crate) fn value<T>(&mut self, index: i64) -> Result<Option<T>, DbError> {
        todo!()
    }
}
