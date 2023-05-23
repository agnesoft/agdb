pub mod indexed_map_impl;

use self::indexed_map_impl::IndexedMapImpl;
use super::map::map_data_memory::MapDataMemory;
use super::map::map_impl::MapImpl;
use super::map::multi_map_impl::MultiMapImpl;
use crate::utilities::stable_hash::StableHash;
use std::hash::Hash;
use std::marker::PhantomData;

pub type IndexedMap<K, T> = IndexedMapImpl<K, T, MapDataMemory<K, T>, MapDataMemory<T, K>>;

impl<K, T> IndexedMap<K, T>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash,
    T: Clone + Default + Eq + Hash + PartialEq + StableHash,
{
    pub fn new() -> Self {
        Self {
            keys_to_values: MapImpl {
                multi_map: MultiMapImpl {
                    data: MapDataMemory::<K, T>::new(),
                    phantom_marker: PhantomData,
                },
            },
            values_to_keys: MapImpl {
                multi_map: MultiMapImpl {
                    data: MapDataMemory::<T, K>::new(),
                    phantom_marker: PhantomData,
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = IndexedMap::<String, u64>::new();
        let key = "alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&key, &value), Ok(()));

        assert_eq!(map.value(&key), Ok(Some(value)));
        assert_eq!(map.key(&value), Ok(Some(key)));
    }

    #[test]
    fn iter() {
        let mut map = IndexedMap::<String, u64>::new();
        assert_eq!(map.insert(&"alias1".to_string(), &1_u64), Ok(()));
        assert_eq!(map.insert(&"alias2".to_string(), &2_u64), Ok(()));
        assert_eq!(map.insert(&"alias3".to_string(), &3_u64), Ok(()));

        let mut values = Vec::<(String, u64)>::new();

        for key_value in map.iter() {
            values.push(key_value);
        }

        values.sort();

        assert_eq!(
            values,
            vec![
                ("alias1".to_string(), 1_u64),
                ("alias2".to_string(), 2_u64),
                ("alias3".to_string(), 3_u64)
            ]
        );
    }

    #[test]
    fn replace_by_key() {
        let mut map = IndexedMap::<String, u64>::new();
        let key = "alias".to_string();
        let value = 1_u64;
        let new_value = 2_u64;

        assert_eq!(map.insert(&key, &value), Ok(()));
        assert_eq!(map.insert(&key, &new_value), Ok(()));

        assert_eq!(map.value(&key), Ok(Some(new_value)));
        assert_eq!(map.key(&new_value), Ok(Some(key)));
        assert_eq!(map.key(&value), Ok(None));
    }

    #[test]
    fn replace_by_value() {
        let mut map = IndexedMap::<String, u64>::new();
        let key = "alias".to_string();
        let new_key = "new_alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&key, &value), Ok(()));
        assert_eq!(map.insert(&new_key, &value), Ok(()));

        assert_eq!(map.value(&key), Ok(None));
        assert_eq!(map.value(&new_key), Ok(Some(value)));
        assert_eq!(map.key(&value), Ok(Some(new_key)));
    }

    #[test]
    fn remove_key() {
        let mut map = IndexedMap::<String, u64>::new();
        let key = "alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&key, &value), Ok(()));

        assert_eq!(map.value(&key), Ok(Some(value)));
        assert_eq!(map.key(&value), Ok(Some(key.clone())));

        map.remove_key(&key).unwrap();
        map.remove_key(&key).unwrap();

        assert_eq!(map.value(&key), Ok(None));
        assert_eq!(map.key(&value), Ok(None));
    }

    #[test]
    fn remove_value() {
        let mut map = IndexedMap::<String, u64>::new();
        let key = "alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&key, &value), Ok(()));

        assert_eq!(map.value(&key), Ok(Some(value)));
        assert_eq!(map.key(&value), Ok(Some(key.clone())));

        map.remove_value(&value).unwrap();
        map.remove_value(&value).unwrap();

        assert_eq!(map.value(&key), Ok(None));
        assert_eq!(map.key(&value), Ok(None));
    }

    #[test]
    fn value() {
        let mut map = IndexedMap::<i64, i64>::new();
        map.insert(&1, &1).unwrap();
        map.insert(&2, &2).unwrap();
        map.insert(&-1, &-3).unwrap();

        assert_eq!(map.value(&-1), Ok(Some(-3)));
    }
}
