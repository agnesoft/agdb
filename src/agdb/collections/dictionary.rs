pub mod dictionary_data_storage;
pub mod dictionary_data_storage_indexes;
pub mod dictionary_impl;
pub mod dictionary_index;
pub mod dictionary_value;

mod dictionary_data;
mod dictionary_data_memory;

use self::dictionary_data_memory::DictionaryDataMemory;
use self::dictionary_impl::DictionaryImpl;
use crate::utilities::serialize::OldSerialize;
use crate::utilities::stable_hash::StableHash;
use std::marker::PhantomData;

pub type Dictionary<T> = DictionaryImpl<T, DictionaryDataMemory<T>>;

impl<T> Dictionary<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + OldSerialize,
{
    pub fn new() -> Dictionary<T> {
        Dictionary {
            data: DictionaryDataMemory::<T>::default(),
            phantom_data: PhantomData,
        }
    }
}

impl<T> Default for Dictionary<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + OldSerialize,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::dictionary::dictionary_index::DictionaryIndex;
    use super::*;
    use crate::test_utilities::collided_value::CollidedValue;

    #[test]
    fn count_invalid_index() {
        let dictionary = Dictionary::<i64>::new();

        assert_eq!(dictionary.count(&DictionaryIndex::default()), Ok(None));
        assert_eq!(dictionary.count(&DictionaryIndex::from(-1_i64)), Ok(None));
    }

    #[test]
    fn derived_from_default() {
        let _dictionary = Dictionary::<i64>::default();
    }

    #[test]
    fn index() {
        let mut dictionary = Dictionary::<i64>::new();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.index(&10_i64), Ok(Some(index)));
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
        dictionary.remove(&index).unwrap();

        assert_eq!(dictionary.index(&10), Ok(None));
    }

    #[test]
    fn index_reuse() {
        let mut dictionary = Dictionary::<i64>::new();

        let index1 = dictionary.insert(&5).unwrap();
        let index2 = dictionary.insert(&10).unwrap();
        let index3 = dictionary.insert(&7).unwrap();

        dictionary.remove(&index2).unwrap();
        dictionary.remove(&index1).unwrap();
        dictionary.remove(&index3).unwrap();

        assert_eq!(dictionary.count(&index1), Ok(None));
        assert_eq!(dictionary.count(&index2), Ok(None));
        assert_eq!(dictionary.count(&index3), Ok(None));

        assert_eq!(dictionary.insert(&3), Ok(index3.clone()));
        assert_eq!(dictionary.insert(&2), Ok(index1.clone()));
        assert_eq!(dictionary.insert(&1), Ok(index2.clone()));

        assert_eq!(dictionary.value(&index1), Ok(Some(2)));
        assert_eq!(dictionary.value(&index2), Ok(Some(1)));
        assert_eq!(dictionary.value(&index3), Ok(Some(3)));
    }

    #[test]
    fn index_with_collisions() {
        let mut dictionary = Dictionary::<CollidedValue<i64>>::new();

        let index1 = dictionary.insert(&CollidedValue::new(1)).unwrap();
        let index2 = dictionary.insert(&CollidedValue::new(2)).unwrap();
        let index3 = dictionary.insert(&CollidedValue::new(3)).unwrap();

        assert_eq!(dictionary.index(&CollidedValue::new(1)), Ok(Some(index1)));
        assert_eq!(dictionary.index(&CollidedValue::new(2)), Ok(Some(index2)));
        assert_eq!(dictionary.index(&CollidedValue::new(3)), Ok(Some(index3)));
    }

    #[test]
    fn insert() {
        let mut dictionary = Dictionary::<i64>::new();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.len(), Ok(1));
        assert_eq!(dictionary.value(&index), Ok(Some(10_i64)));
        assert_eq!(dictionary.count(&index), Ok(Some(1)));
    }

    #[test]
    fn insert_multiple() {
        let mut dictionary = Dictionary::<i64>::new();

        let index1 = dictionary.insert(&10).unwrap();
        let index2 = dictionary.insert(&15).unwrap();
        let index3 = dictionary.insert(&20).unwrap();

        assert_eq!(dictionary.len(), Ok(3));

        assert_eq!(dictionary.value(&index1).unwrap(), Some(10_i64));
        assert_eq!(dictionary.count(&index1), Ok(Some(1)));

        assert_eq!(dictionary.value(&index2).unwrap(), Some(15_i64));
        assert_eq!(dictionary.count(&index2), Ok(Some(1)));

        assert_eq!(dictionary.value(&index3).unwrap(), Some(20_i64));
        assert_eq!(dictionary.count(&index3), Ok(Some(1)));
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
        assert_eq!(dictionary.count(&index2), Ok(Some(3)));
    }

    #[test]
    fn remove() {
        let mut dictionary = Dictionary::<i64>::new();

        let index = dictionary.insert(&10).unwrap();
        dictionary.remove(&index).unwrap();

        assert_eq!(dictionary.value(&index), Ok(None));
        assert_eq!(dictionary.count(&index), Ok(None));
    }

    #[test]
    fn remove_duplicated() {
        let mut dictionary = Dictionary::<i64>::new();

        let index = dictionary.insert(&10).unwrap();
        dictionary.insert(&10).unwrap();
        dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.value(&index), Ok(Some(10)));
        assert_eq!(dictionary.count(&index), Ok(Some(3)));

        dictionary.remove(&index).unwrap();

        assert_eq!(dictionary.value(&index), Ok(Some(10)));
        assert_eq!(dictionary.count(&index), Ok(Some(2)));

        dictionary.remove(&index).unwrap();
        dictionary.remove(&index).unwrap();

        assert_eq!(dictionary.value(&index), Ok(None));
        assert_eq!(dictionary.count(&index), Ok(None));
    }

    #[test]
    fn remove_missing() {
        let mut dictionary = Dictionary::<i64>::new();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.len(), Ok(1));

        dictionary
            .remove(&DictionaryIndex::from(index.value() + 1))
            .unwrap();

        assert_eq!(dictionary.len(), Ok(1));
    }

    #[test]
    fn value_missing_index() {
        let dictionary = Dictionary::<i64>::new();
        assert_eq!(dictionary.value(&DictionaryIndex::from(1_i64)), Ok(None));
    }
}
