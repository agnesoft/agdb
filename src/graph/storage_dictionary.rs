use super::dictionary_data_storage::DictionaryDataStorage;
use super::dictionary_impl::DictionaryImpl;

pub(crate) type StorageDictionary<T> = DictionaryImpl<T, DictionaryDataStorage<T>>;
