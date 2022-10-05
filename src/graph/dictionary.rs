use super::dictionary_data_memory::DictionaryDataMemory;
use super::dictionary_impl::DictionaryImpl;
use crate::storage::HashMultiMap;
use crate::storage::Serialize;
use crate::storage::StableHash;

pub(crate) type Dictionary<T> = DictionaryImpl<T, DictionaryDataMemory<T>>;

impl<T> Dictionary<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    pub(crate) fn new() -> Dictionary<T> {
        Dictionary {
            data: DictionaryDataMemory::<T> {
                index: HashMultiMap::<u64, i64>::new(),
                values: vec![],
            },
            phantom_data: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index() {}

    #[test]
    fn insert() {}

    #[test]
    fn remove() {}

    #[test]
    fn value() {}
}
