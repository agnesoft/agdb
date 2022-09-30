use super::dictionary_data::DictionaryData;
use crate::DbError;

pub(crate) struct DictionaryImpl<Data: DictionaryData> {
    data: Data,
}

impl<Data: DictionaryData> DictionaryImpl<Data> {
    pub(crate) fn insert<T>(&mut self, index: i64, value: T) -> Result<(), DbError> {
        Ok(())
    }
}
