use super::dictionary_data_memory::DictionaryDataMemory;
use super::dictionary_impl::DictionaryImpl;
use super::dictionary_value::DictionaryValue;
use crate::storage::HashMultiMap;
use crate::storage::StableHash;
use agdb_serialize::Serialize;
use std::marker::PhantomData;

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
            phantom_data: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Default, Eq, PartialEq)]
    struct CollidedValue {
        pub(super) value: i64,
    }

    impl Serialize for CollidedValue {
        fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
            Ok(CollidedValue {
                value: i64::deserialize(bytes)?,
            })
        }

        fn serialize(&self) -> Vec<u8> {
            self.value.serialize()
        }
    }

    impl StableHash for CollidedValue {
        fn stable_hash(&self) -> u64 {
            1
        }
    }

    #[test]
    fn collided_value() {
        let value = CollidedValue { value: 10 };
        let bytes = value.serialize();
        let other = CollidedValue::deserialize(&bytes).unwrap();

        assert!(value == other);
    }

    #[test]
    fn count_invalid_index() {
        let dictionary = Dictionary::<i64>::new();

        assert_eq!(dictionary.count(-1), Ok(None));
    }

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
    fn index_reuse() {
        let mut dictionary = Dictionary::<i64>::new();

        let index1 = dictionary.insert(&5).unwrap();
        let index2 = dictionary.insert(&10).unwrap();
        let index3 = dictionary.insert(&7).unwrap();

        dictionary.remove(index2).unwrap();
        dictionary.remove(index1).unwrap();
        dictionary.remove(index3).unwrap();

        assert_eq!(dictionary.count(index1), Ok(None));
        assert_eq!(dictionary.count(index2), Ok(None));
        assert_eq!(dictionary.count(index3), Ok(None));

        assert_eq!(dictionary.insert(&3), Ok(index3));
        assert_eq!(dictionary.insert(&2), Ok(index1));
        assert_eq!(dictionary.insert(&1), Ok(index2));

        assert_eq!(dictionary.value(index1), Ok(Some(2)));
        assert_eq!(dictionary.value(index2), Ok(Some(1)));
        assert_eq!(dictionary.value(index3), Ok(Some(3)));
    }

    #[test]
    fn index_with_collisions() {
        let mut dictionary = Dictionary::<CollidedValue>::new();

        let index1 = dictionary.insert(&CollidedValue { value: 1 }).unwrap();
        let index2 = dictionary.insert(&CollidedValue { value: 2 }).unwrap();
        let index3 = dictionary.insert(&CollidedValue { value: 3 }).unwrap();

        assert_eq!(
            dictionary.index(&CollidedValue { value: 1 }),
            Ok(Some(index1))
        );

        assert_eq!(
            dictionary.index(&CollidedValue { value: 2 }),
            Ok(Some(index2))
        );

        assert_eq!(
            dictionary.index(&CollidedValue { value: 3 }),
            Ok(Some(index3))
        );
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
        let dictionary = Dictionary::<i64>::new();

        assert_eq!(dictionary.value(1), Ok(None));
    }
}
