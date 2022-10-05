use super::dictionary_data_memory::DictionaryDataMemory;
use super::dictionary_impl::DictionaryImpl;
use super::dictionary_value::DictionaryValue;
use crate::storage::HashMultiMap;
use crate::storage::Serialize;
use crate::storage::StableHash;

pub(crate) type Dictionary<T> = DictionaryImpl<T, DictionaryDataMemory<T>>;

#[allow(dead_code)]
impl<T> Dictionary<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
{
    pub(crate) fn new() -> Dictionary<T> {
        Dictionary {
            data: DictionaryDataMemory::<T> {
                index: HashMultiMap::<u64, i64>::new(),
                values: vec![DictionaryValue::<T> {
                    meta: 0,
                    hash: 0,
                    value: T::default(),
                }],
            },
            phantom_data: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index() {
        let mut dictionary = Dictionary::<i64>::new();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.index(&10), Ok(Some(index)));
    }

    #[test]
    fn index_missing_value() {
        let dictionary = Dictionary::<i64>::new();

        assert_eq!(dictionary.index(&10), Ok(None));
    }

    #[test]
    fn index_removed_value() {
        let mut dictionary = Dictionary::<i64>::new();

        let index = dictionary.insert(&10).unwrap();
        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.index(&10), Ok(None));
    }

    #[test]
    fn insert() {
        let mut dictionary = Dictionary::<i64>::new();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.len(), Ok(1));
        assert_eq!(dictionary.value(index), Ok(Some(10_i64)));
        assert_eq!(dictionary.count(index), Ok(Some(1)));
    }

    #[test]
    fn insert_multiple() {
        let mut dictionary = Dictionary::<i64>::new();

        let index1 = dictionary.insert(&10).unwrap();
        let index2 = dictionary.insert(&15).unwrap();
        let index3 = dictionary.insert(&20).unwrap();

        assert_eq!(dictionary.len(), Ok(3));

        assert_eq!(dictionary.value(index1).unwrap(), Some(10_i64));
        assert_eq!(dictionary.count(index1), Ok(Some(1)));

        assert_eq!(dictionary.value(index2).unwrap(), Some(15_i64));
        assert_eq!(dictionary.count(index2), Ok(Some(1)));

        assert_eq!(dictionary.value(index3).unwrap(), Some(20_i64));
        assert_eq!(dictionary.count(index3), Ok(Some(1)));
    }

    #[test]
    fn insert_same() {
        let mut dictionary = Dictionary::<i64>::new();

        dictionary.insert(&10).unwrap();

        let index2 = dictionary.insert(&15).unwrap();

        assert_eq!(dictionary.insert(&15).unwrap(), index2);
        assert_eq!(dictionary.insert(&15).unwrap(), index2);

        dictionary.insert(&20).unwrap();

        assert_eq!(dictionary.len(), Ok(3));
        assert_eq!(dictionary.count(index2), Ok(Some(3)));
    }

    #[test]
    fn remove() {
        let mut dictionary = Dictionary::<i64>::new();

        let index = dictionary.insert(&10).unwrap();
        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.value(index), Ok(None));
        assert_eq!(dictionary.count(index), Ok(None));
    }

    #[test]
    fn remove_duplicated() {
        let mut dictionary = Dictionary::<i64>::new();

        let index = dictionary.insert(&10).unwrap();
        dictionary.insert(&10).unwrap();
        dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.value(index), Ok(Some(10)));
        assert_eq!(dictionary.count(index), Ok(Some(3)));

        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.value(index), Ok(Some(10)));
        assert_eq!(dictionary.count(index), Ok(Some(2)));

        dictionary.remove(index).unwrap();
        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.value(index), Ok(None));
        assert_eq!(dictionary.count(index), Ok(None));
    }

    #[test]
    fn remove_missing() {
        let mut dictionary = Dictionary::<i64>::new();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.len(), Ok(1));

        dictionary.remove(index + 1).unwrap();

        assert_eq!(dictionary.len(), Ok(1));
    }

    #[test]
    fn value_missing_index() {
        let mut dictionary = Dictionary::<i64>::new();

        assert_eq!(dictionary.value(1), Ok(None));
    }
}
