use super::map::DbMap;
use super::map::DbMapData;
use super::map::MapData;
use super::map::MapImpl;
use super::map::MapIterator;
use super::vec::VecValue;
use crate::db::db_error::DbError;
use crate::storage::file_storage::FileStorage;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::stable_hash::StableHash;
use std::marker::PhantomData;

pub struct IndexedMapImpl<K, T, S, DataKT, DataTK>
where
    DataKT: MapData<K, T, S>,
    DataTK: MapData<T, K, S>,
{
    pub(crate) keys_to_values: MapImpl<K, T, S, DataKT>,
    pub(crate) values_to_keys: MapImpl<T, K, S, DataTK>,
    pub(crate) storage: PhantomData<S>,
}

impl<K, T, S, DataKT, DataTK> IndexedMapImpl<K, T, S, DataKT, DataTK>
where
    K: Default + PartialEq + StableHash,
    T: Default + PartialEq + StableHash,
    DataKT: MapData<K, T, S>,
    DataTK: MapData<T, K, S>,
{
    pub fn insert(&mut self, storage: &mut S, key: &K, value: &T) -> Result<(), DbError> {
        if let Some(v) = self.keys_to_values.insert(storage, key, value)? {
            self.values_to_keys.remove(storage, &v)?;
        }

        if let Some(k) = self.values_to_keys.insert(storage, value, key)? {
            self.keys_to_values.remove(storage, &k)?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn iter(&self) -> MapIterator<K, T, S, DataKT> {
        self.keys_to_values.iter()
    }

    pub fn key(&self, value: &T) -> Result<Option<K>, DbError> {
        self.values_to_keys.value(value)
    }

    pub fn remove_key(&mut self, storage: &mut S, key: &K) -> Result<(), DbError> {
        if let Some(value) = self.keys_to_values.value(key)? {
            self.values_to_keys.remove(storage, &value)?;
        }

        self.keys_to_values.remove(storage, key)
    }

    #[allow(dead_code)]
    pub fn remove_value(&mut self, storage: &mut S, value: &T) -> Result<(), DbError> {
        if let Some(key) = self.values_to_keys.value(value)? {
            self.keys_to_values.remove(storage, &key)?;
        }

        self.values_to_keys.remove(storage, value)
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        self.keys_to_values.value(key)
    }
}

pub type DbIndexedMap<K, T, S = FileStorage> =
    IndexedMapImpl<K, T, S, DbMapData<K, T, S>, DbMapData<T, K, S>>;

impl<K, T, S> DbIndexedMap<K, T, S>
where
    K: Default + Clone + VecValue,
    T: Default + Clone + VecValue,
    S: Storage,
{
    pub fn new(storage: &mut S) -> Result<Self, DbError> {
        let keys_to_values = DbMap::<K, T, S>::new(storage)?;
        let values_to_keys = DbMap::<T, K, S>::new(storage)?;

        Ok(Self {
            keys_to_values,
            values_to_keys,
            storage: PhantomData,
        })
    }

    pub fn from_storage(storage: &S, index: (StorageIndex, StorageIndex)) -> Result<Self, DbError> {
        let keys_to_values = DbMap::<K, T, S>::from_storage(storage, index.0)?;
        let values_to_keys = DbMap::<T, K, S>::from_storage(storage, index.1)?;

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
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn from_storage() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();

        let storage_index;

        {
            let mut map = DbIndexedMap::<String, u64>::new(&mut storage).unwrap();
            let key = "alias".to_string();
            let value = 1_u64;
            map.insert(&mut storage, &key, &value).unwrap();
            storage_index = map.storage_index();
        }

        let map = DbIndexedMap::<String, u64>::from_storage(&storage, storage_index).unwrap();
        assert_eq!(map.value(&"alias".to_string()).unwrap(), Some(1_u64));
    }

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut map = DbIndexedMap::<String, u64>::new(&mut storage).unwrap();
        let key = "alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&mut storage, &key, &value), Ok(()));

        assert_eq!(map.value(&key), Ok(Some(value)));
        assert_eq!(map.key(&value), Ok(Some(key)));
    }

    #[test]
    fn iter() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut map = DbIndexedMap::<String, u64>::new(&mut storage).unwrap();
        assert_eq!(
            map.insert(&mut storage, &"alias1".to_string(), &1_u64),
            Ok(())
        );
        assert_eq!(
            map.insert(&mut storage, &"alias2".to_string(), &2_u64),
            Ok(())
        );
        assert_eq!(
            map.insert(&mut storage, &"alias3".to_string(), &3_u64),
            Ok(())
        );

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
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut map = DbIndexedMap::<String, u64>::new(&mut storage).unwrap();
        let key = "alias".to_string();
        let value = 1_u64;
        let new_value = 2_u64;

        assert_eq!(map.insert(&mut storage, &key, &value), Ok(()));
        assert_eq!(map.insert(&mut storage, &key, &new_value), Ok(()));

        assert_eq!(map.value(&key), Ok(Some(new_value)));
        assert_eq!(map.key(&new_value), Ok(Some(key)));
        assert_eq!(map.key(&value), Ok(None));
    }

    #[test]
    fn replace_by_value() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut map = DbIndexedMap::<String, u64>::new(&mut storage).unwrap();
        let key = "alias".to_string();
        let new_key = "new_alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&mut storage, &key, &value), Ok(()));
        assert_eq!(map.insert(&mut storage, &new_key, &value), Ok(()));

        assert_eq!(map.value(&key), Ok(None));
        assert_eq!(map.value(&new_key), Ok(Some(value)));
        assert_eq!(map.key(&value), Ok(Some(new_key)));
    }

    #[test]
    fn remove_key() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut map = DbIndexedMap::<String, u64>::new(&mut storage).unwrap();
        let key = "alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&mut storage, &key, &value), Ok(()));

        assert_eq!(map.value(&key), Ok(Some(value)));
        assert_eq!(map.key(&value), Ok(Some(key.clone())));

        map.remove_key(&mut storage, &key).unwrap();
        map.remove_key(&mut storage, &key).unwrap();

        assert_eq!(map.value(&key), Ok(None));
        assert_eq!(map.key(&value), Ok(None));
    }

    #[test]
    fn remove_value() {
        let test_file = TestFile::new();
        let mut storage = FileStorage::new(test_file.file_name()).unwrap();
        let mut map = DbIndexedMap::<String, u64>::new(&mut storage).unwrap();
        let key = "alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&mut storage, &key, &value), Ok(()));

        assert_eq!(map.value(&key), Ok(Some(value)));
        assert_eq!(map.key(&value), Ok(Some(key.clone())));

        map.remove_value(&mut storage, &value).unwrap();
        map.remove_value(&mut storage, &value).unwrap();

        assert_eq!(map.value(&key), Ok(None));
        assert_eq!(map.key(&value), Ok(None));
    }
}
