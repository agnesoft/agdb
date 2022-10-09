use super::dictionary_data_memory::DictionaryDataMemory;
use super::dictionary_impl::DictionaryImpl;
use agdb_serialize::Serialize;
use agdb_utilities::StableHash;
use std::marker::PhantomData;

pub type Dictionary<T> = DictionaryImpl<T, DictionaryDataMemory<T>>;

impl<T> Dictionary<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    pub fn new() -> Dictionary<T> {
        Dictionary {
            data: DictionaryDataMemory::<T>::default(),
            phantom_data: PhantomData,
        }
    }
}
