use super::dictionary_data::DictionaryData;
use super::dictionary_value::DictionaryValue;
use crate::storage::HashMultiMap;
use crate::storage::Serialize;
use crate::storage::StableHash;

pub(crate) struct DictionaryDataMemory<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    pub(super) index: HashMultiMap<u64, i64>,
    pub(super) values: Vec<DictionaryValue<T>>,
}

impl<T> DictionaryData<T> for DictionaryDataMemory<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    fn commit(&mut self) -> Result<(), crate::DbError> {
        todo!()
    }

    fn transaction(&mut self) {
        todo!()
    }
}
