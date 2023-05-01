use super::indexed_map::indexed_map_impl::IndexedMapImpl;
use super::indexed_map::IndexedMap;
use super::map::map_data_memory::MapDataMemory;
use super::map::map_data_storage::MapDataStorage;
use super::map::map_impl::MapImpl;
use super::map::multi_map_impl::MultiMapImpl;
use super::map_storage::MapStorage;
use crate::db::db_error::DbError;
use crate::storage::file_storage::FileStorage;
use crate::storage::storage_index::StorageIndex;
use crate::storage::storage_value::StorageValue;
use crate::storage::Storage;
use crate::utilities::stable_hash::StableHash;
use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

type IndexedMapStorage<K, T, Data = FileStorage> =
    IndexedMapImpl<K, T, MapDataStorage<K, T, Data>, MapDataStorage<T, K, Data>>;

#[allow(dead_code)]
impl<K, T, Data> IndexedMapStorage<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + StorageValue,
    T: Clone + Default + Eq + Hash + PartialEq + StableHash + StorageValue,
    Data: Storage,
{
    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        let keys_to_values = MapStorage::<K, T, Data>::new(storage.clone())?;
        let values_to_keys = MapStorage::<T, K, Data>::new(storage)?;

        Ok(Self {
            keys_to_values,
            values_to_keys,
        })
    }

    pub fn from_storage(
        storage: Rc<RefCell<Data>>,
        index: &(StorageIndex, StorageIndex),
    ) -> Result<Self, DbError> {
        let keys_to_values = MapStorage::<K, T, Data>::from_storage(storage.clone(), &index.0)?;
        let values_to_keys = MapStorage::<T, K, Data>::from_storage(storage, &index.1)?;

        Ok(Self {
            keys_to_values,
            values_to_keys,
        })
    }

    pub fn storage_index(&self) -> (StorageIndex, StorageIndex) {
        (
            self.keys_to_values.storage_index(),
            self.values_to_keys.storage_index(),
        )
    }

    pub fn to_indexed_map(&self) -> Result<IndexedMap<K, T>, DbError> {
        let mut keys_to_values = MapImpl::<K, T, MapDataMemory<K, T>> {
            multi_map: MultiMapImpl {
                data: MapDataMemory::<K, T>::new(),
                phantom_marker: PhantomData,
            },
        };

        for (key, value) in self.keys_to_values.iter() {
            keys_to_values.insert(&key, &value)?;
        }

        let mut values_to_keys = MapImpl::<T, K, MapDataMemory<T, K>> {
            multi_map: MultiMapImpl {
                data: MapDataMemory::<T, K>::new(),
                phantom_marker: PhantomData,
            },
        };

        for (key, value) in self.values_to_keys.iter() {
            values_to_keys.insert(&key, &value)?;
        }

        Ok(IndexedMap {
            keys_to_values,
            values_to_keys,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn from_storage() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let index;

        {
            let mut map = IndexedMapStorage::<String, u64>::new(storage.clone()).unwrap();
            let key = "alias".to_string();
            let value = 1_u64;
            map.insert(&key, &value).unwrap();
            index = map.storage_index();
        }

        let map = IndexedMapStorage::<String, u64>::from_storage(storage, &index).unwrap();
        assert_eq!(map.value(&"alias".to_string()).unwrap(), Some(1_u64));
    }

    #[test]
    fn to_indexed_map() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = IndexedMapStorage::<String, u64>::new(storage).unwrap();
        map.insert(&"alias".to_string(), &1_u64).unwrap();

        let mem_map = map.to_indexed_map().unwrap();

        assert_eq!(mem_map.value(&"alias".to_string()).unwrap(), Some(1_u64));
    }

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = IndexedMapStorage::<String, u64>::new(storage).unwrap();
        let key = "alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&key, &value), Ok(()));

        assert_eq!(map.value(&key), Ok(Some(value)));
        assert_eq!(map.key(&value), Ok(Some(key)));
    }

    #[test]
    fn iter() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = IndexedMapStorage::<String, u64>::new(storage).unwrap();
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
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = IndexedMapStorage::<String, u64>::new(storage).unwrap();
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
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = IndexedMapStorage::<String, u64>::new(storage).unwrap();
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
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = IndexedMapStorage::<String, u64>::new(storage).unwrap();
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
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = IndexedMapStorage::<String, u64>::new(storage).unwrap();
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
}
