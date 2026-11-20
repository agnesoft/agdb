use crate::DbError;
use crate::StorageData;
use crate::collections::map::DbMap;
use crate::collections::map::DbMapData;
use crate::collections::map::MapData;
use crate::collections::map::MapImpl;
use crate::collections::map::MapIterator;
use crate::collections::vec::VecValue;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::stable_hash::StableHash;
use std::marker::PhantomData;

pub struct IndexedMapImpl<K, T, D, DataKT, DataTK>
where
    D: StorageData,
    DataKT: MapData<K, T, D>,
    DataTK: MapData<T, K, D>,
{
    keys_to_values: MapImpl<K, T, D, DataKT>,
    values_to_keys: MapImpl<T, K, D, DataTK>,
    storage: PhantomData<D>,
}

impl<K, T, D, DataKT, DataTK> IndexedMapImpl<K, T, D, DataKT, DataTK>
where
    D: StorageData,
    K: Default + PartialEq + StableHash,
    T: Default + PartialEq + StableHash,
    DataKT: MapData<K, T, D>,
    DataTK: MapData<T, K, D>,
{
    pub fn insert(&mut self, storage: &mut Storage<D>, key: &K, value: &T) -> Result<(), DbError> {
        if let Some(v) = self.keys_to_values.insert(storage, key, value)? {
            self.values_to_keys.remove(storage, &v)?;
        }

        if let Some(k) = self.values_to_keys.insert(storage, value, key)? {
            self.keys_to_values.remove(storage, &k)?;
        }

        Ok(())
    }

    pub fn iter<'a>(&'a self, storage: &'a Storage<D>) -> MapIterator<'a, K, T, D, DataKT> {
        self.keys_to_values.iter(storage)
    }

    pub fn key(&self, storage: &Storage<D>, value: &T) -> Result<Option<K>, DbError> {
        self.values_to_keys.value(storage, value)
    }

    pub fn remove_key(&mut self, storage: &mut Storage<D>, key: &K) -> Result<(), DbError> {
        if let Some(value) = self.keys_to_values.value(storage, key)? {
            self.values_to_keys.remove(storage, &value)?;
        }

        self.keys_to_values.remove(storage, key)
    }

    #[allow(dead_code)]
    pub fn remove_value(&mut self, storage: &mut Storage<D>, value: &T) -> Result<(), DbError> {
        if let Some(key) = self.values_to_keys.value(storage, value)? {
            self.keys_to_values.remove(storage, &key)?;
        }

        self.values_to_keys.remove(storage, value)
    }

    pub fn shrink_to_fit(&mut self, storage: &mut Storage<D>) -> Result<(), DbError> {
        self.keys_to_values.shrink_to_fit(storage)?;
        self.values_to_keys.shrink_to_fit(storage)
    }

    pub fn value(&self, storage: &Storage<D>, key: &K) -> Result<Option<T>, DbError> {
        self.keys_to_values.value(storage, key)
    }
}

pub type DbIndexedMap<K, T, D> = IndexedMapImpl<K, T, D, DbMapData<K, T, D>, DbMapData<T, K, D>>;

impl<K, T, D> DbIndexedMap<K, T, D>
where
    K: Default + Clone + VecValue<D>,
    T: Default + Clone + VecValue<D>,
    D: StorageData,
{
    pub fn new(storage: &mut Storage<D>) -> Result<Self, DbError> {
        let keys_to_values = DbMap::<K, T, D>::new(storage)?;
        let values_to_keys = DbMap::<T, K, D>::new(storage)?;

        Ok(Self {
            keys_to_values,
            values_to_keys,
            storage: PhantomData,
        })
    }

    pub fn from_storage(
        storage: &Storage<D>,
        index: (StorageIndex, StorageIndex),
    ) -> Result<Self, DbError> {
        let keys_to_values = DbMap::<K, T, D>::from_storage(storage, index.0)?;
        let values_to_keys = DbMap::<T, K, D>::from_storage(storage, index.1)?;

        Ok(Self {
            keys_to_values,
            values_to_keys,
            storage: PhantomData,
        })
    }

    pub fn storage_index(&self) -> (StorageIndex, StorageIndex) {
        (
            self.keys_to_values.storage_index(),
            self.values_to_keys.storage_index(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::tests::TestStorage;
    use crate::collections::tests::add_spare_capacity;
    use crate::collections::tests::assert_io_failures;
    use crate::collections::tests::storage;
    use crate::storage::file_storage::FileStorageImpl;
    use agplatform::test_platform::TestPlatform;

    type TestMap = DbIndexedMap<String, u64, FileStorageImpl<TestPlatform>>;

    fn map(storage: &mut TestStorage, pairs: &[(&str, u64)]) -> TestMap {
        let mut map = TestMap::new(storage).unwrap();

        for (key, value) in pairs {
            map.insert(storage, &key.to_string(), value).unwrap();
        }

        map
    }

    fn sorted(map: &TestMap, storage: &TestStorage) -> Vec<(String, u64)> {
        let mut pairs = map.iter(storage).collect::<Vec<_>>();
        pairs.sort();
        pairs
    }

    fn pairs(pairs: &[(&str, u64)]) -> Vec<(String, u64)> {
        pairs
            .iter()
            .map(|(key, value)| (key.to_string(), *value))
            .collect()
    }

    mod indexed_map_impl {
        use super::*;

        mod insert {
            use super::*;

            #[test]
            fn insert_should_map_both_ways() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[]);

                map.insert(&mut storage, &"a".to_string(), &1).unwrap();

                assert_eq!(map.value(&storage, &"a".to_string()), Ok(Some(1)));
                assert_eq!(map.key(&storage, &1), Ok(Some("a".to_string())));
            }

            #[test]
            fn insert_existing_key_should_drop_its_old_value() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[("a", 1)]);

                map.insert(&mut storage, &"a".to_string(), &2).unwrap();

                assert_eq!(map.key(&storage, &1), Ok(None));
                assert_eq!(map.key(&storage, &2), Ok(Some("a".to_string())));
                assert_eq!(sorted(&map, &storage), pairs(&[("a", 2)]));
            }

            #[test]
            fn insert_existing_value_should_drop_its_old_key() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[("a", 1)]);

                map.insert(&mut storage, &"b".to_string(), &1).unwrap();

                assert_eq!(map.value(&storage, &"a".to_string()), Ok(None));
                assert_eq!(map.key(&storage, &1), Ok(Some("b".to_string())));
                assert_eq!(sorted(&map, &storage), pairs(&[("b", 1)]));
            }

            #[test]
            fn insert_existing_key_and_value_should_drop_both_old_pairs() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[("a", 1), ("b", 2)]);

                map.insert(&mut storage, &"a".to_string(), &2).unwrap();

                assert_eq!(map.key(&storage, &1), Ok(None));
                assert_eq!(map.value(&storage, &"b".to_string()), Ok(None));
                assert_eq!(sorted(&map, &storage), pairs(&[("a", 2)]));
            }

            #[test]
            fn insert_same_pair_should_keep_it() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[("a", 1)]);

                map.insert(&mut storage, &"a".to_string(), &1).unwrap();

                assert_eq!(map.key(&storage, &1), Ok(Some("a".to_string())));
                assert_eq!(sorted(&map, &storage), pairs(&[("a", 1)]));
            }

            #[test]
            fn insert_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[("a", 1), ("b", 2)]),
                    |storage, map| map.insert(storage, &"a".to_string(), &2),
                );
            }
        }

        mod iter {
            use super::*;

            #[test]
            fn iter_should_yield_all_pairs() {
                let mut storage = storage();
                let map = map(&mut storage, &[("a", 1), ("b", 2), ("c", 3)]);

                assert_eq!(
                    sorted(&map, &storage),
                    pairs(&[("a", 1), ("b", 2), ("c", 3)])
                );
            }
        }

        mod key {
            use super::*;

            #[test]
            fn key_should_return_key_of_value() {
                let mut storage = storage();
                let map = map(&mut storage, &[("a", 1)]);

                assert_eq!(map.key(&storage, &1), Ok(Some("a".to_string())));
                assert_eq!(map.key(&storage, &2), Ok(None));
            }
        }

        mod remove_key {
            use super::*;

            #[test]
            fn remove_key_should_remove_pair() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[("a", 1), ("b", 2)]);

                map.remove_key(&mut storage, &"a".to_string()).unwrap();

                assert_eq!(map.key(&storage, &1), Ok(None));
                assert_eq!(sorted(&map, &storage), pairs(&[("b", 2)]));
            }

            #[test]
            fn remove_key_missing_should_do_nothing() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[("a", 1)]);

                map.remove_key(&mut storage, &"b".to_string()).unwrap();

                assert_eq!(sorted(&map, &storage), pairs(&[("a", 1)]));
            }

            #[test]
            fn remove_key_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[("a", 1)]),
                    |storage, map| map.remove_key(storage, &"a".to_string()),
                );
            }
        }

        mod remove_value {
            use super::*;

            #[test]
            fn remove_value_should_remove_pair() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[("a", 1), ("b", 2)]);

                map.remove_value(&mut storage, &1).unwrap();

                assert_eq!(map.value(&storage, &"a".to_string()), Ok(None));
                assert_eq!(sorted(&map, &storage), pairs(&[("b", 2)]));
            }

            #[test]
            fn remove_value_missing_should_do_nothing() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[("a", 1)]);

                map.remove_value(&mut storage, &2).unwrap();

                assert_eq!(sorted(&map, &storage), pairs(&[("a", 1)]));
            }

            #[test]
            fn remove_value_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[("a", 1)]),
                    |storage, map| map.remove_value(storage, &1),
                );
            }
        }

        mod shrink_to_fit {
            use super::*;

            fn shrunk(storage: &mut TestStorage) -> TestMap {
                let keys = (0..100).map(|i| format!("{i}")).collect::<Vec<_>>();
                let mut map = map(storage, &[]);
                for (value, key) in keys.iter().enumerate() {
                    map.insert(storage, key, &(value as u64)).unwrap();
                }
                for key in &keys[1..] {
                    map.remove_key(storage, key).unwrap();
                }
                add_spare_capacity(storage, map.storage_index().0);
                add_spare_capacity(storage, map.storage_index().1);
                TestMap::from_storage(storage, map.storage_index()).unwrap()
            }

            #[test]
            fn shrink_to_fit_should_keep_pairs() {
                let mut storage = storage();
                let mut map = shrunk(&mut storage);

                map.shrink_to_fit(&mut storage).unwrap();

                assert_eq!(sorted(&map, &storage), pairs(&[("0", 0)]));
            }

            #[test]
            fn shrink_to_fit_failure_should_fail() {
                assert_io_failures(shrunk, |storage, map| map.shrink_to_fit(storage));
            }
        }

        mod value {
            use super::*;

            #[test]
            fn value_should_return_value_of_key() {
                let mut storage = storage();
                let map = map(&mut storage, &[("a", 1)]);

                assert_eq!(map.value(&storage, &"a".to_string()), Ok(Some(1)));
                assert_eq!(map.value(&storage, &"b".to_string()), Ok(None));
            }
        }
    }

    mod db_indexed_map {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn new_failure_should_fail() {
                assert_io_failures(|_| (), |storage, _| TestMap::new(storage));
            }
        }

        mod from_storage {
            use super::*;

            #[test]
            fn from_storage_should_load_both_ways() {
                let mut storage = storage();
                let map = map(&mut storage, &[("a", 1)]);

                let map = TestMap::from_storage(&storage, map.storage_index()).unwrap();

                assert_eq!(map.value(&storage, &"a".to_string()), Ok(Some(1)));
                assert_eq!(map.key(&storage, &1), Ok(Some("a".to_string())));
            }

            #[test]
            fn from_storage_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[("a", 1)]).storage_index(),
                    |storage, index| TestMap::from_storage(storage, *index),
                );
            }
        }
    }
}
