use crate::dictionary_data_memory::DictionaryDataMemory;
use crate::dictionary_index::DictionaryIndex;
use crate::dictionary_value::DictionaryValue;
use agdb_multi_map::MultiMap;
use agdb_serialize::Serialize;
use agdb_utilities::StableHash;

impl<T> Default for DictionaryDataMemory<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    fn default() -> Self {
        Self {
            index: MultiMap::<u64, DictionaryIndex>::new(),
            values: vec![DictionaryValue::<T>::default()],
        }
    }
}
