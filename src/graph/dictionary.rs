use super::dictionary_data_memory::DictionaryDataMemory;
use super::dictionary_impl::DictionaryImpl;

pub(crate) type Dictionary = DictionaryImpl<DictionaryDataMemory>;
