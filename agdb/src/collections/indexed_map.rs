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
    use crate::{
        storage::file_storage_memory_mapped::FileStorageMemoryMapped,
        test_utilities::test_file::TestFile,
    };

    #[test]
    fn from_storage() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let storage_index;

        {
            let mut map =
                DbIndexedMap::<String, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
            let key = "alias".to_string();
            let value = 1_u64;
            map.insert(&mut storage, &key, &value).unwrap();
            storage_index = map.storage_index();
        }

        let map = DbIndexedMap::<String, u64, FileStorageMemoryMapped>::from_storage(
            &storage,
            storage_index,
        )
        .unwrap();
        assert_eq!(
            map.value(&storage, &"alias".to_string()).unwrap(),
            Some(1_u64)
        );
    }

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();
        let mut map =
            DbIndexedMap::<String, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
        let key = "alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&mut storage, &key, &value), Ok(()));

        assert_eq!(map.value(&storage, &key), Ok(Some(value)));
        assert_eq!(map.key(&storage, &value), Ok(Some(key)));
    }

    #[test]
    fn iter() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();
        let mut map =
            DbIndexedMap::<String, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
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

        for key_value in map.iter(&storage) {
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
        let mut storage = Storage::new(test_file.file_name()).unwrap();
        let mut map =
            DbIndexedMap::<String, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
        let key = "alias".to_string();
        let value = 1_u64;
        let new_value = 2_u64;

        assert_eq!(map.insert(&mut storage, &key, &value), Ok(()));
        assert_eq!(map.insert(&mut storage, &key, &new_value), Ok(()));

        assert_eq!(map.value(&storage, &key), Ok(Some(new_value)));
        assert_eq!(map.key(&storage, &new_value), Ok(Some(key)));
        assert_eq!(map.key(&storage, &value), Ok(None));
    }

    #[test]
    fn replace_by_value() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();
        let mut map =
            DbIndexedMap::<String, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
        let key = "alias".to_string();
        let new_key = "new_alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&mut storage, &key, &value), Ok(()));
        assert_eq!(map.insert(&mut storage, &new_key, &value), Ok(()));

        assert_eq!(map.value(&storage, &key), Ok(None));
        assert_eq!(map.value(&storage, &new_key), Ok(Some(value)));
        assert_eq!(map.key(&storage, &value), Ok(Some(new_key)));
    }

    #[test]
    fn remove_key() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();
        let mut map =
            DbIndexedMap::<String, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
        let key = "alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&mut storage, &key, &value), Ok(()));

        assert_eq!(map.value(&storage, &key), Ok(Some(value)));
        assert_eq!(map.key(&storage, &value), Ok(Some(key.clone())));

        map.remove_key(&mut storage, &key).unwrap();
        map.remove_key(&mut storage, &key).unwrap();

        assert_eq!(map.value(&storage, &key), Ok(None));
        assert_eq!(map.key(&storage, &value), Ok(None));
    }

    #[test]
    fn remove_value() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();
        let mut map =
            DbIndexedMap::<String, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
        let key = "alias".to_string();
        let value = 1_u64;

        assert_eq!(map.insert(&mut storage, &key, &value), Ok(()));

        assert_eq!(map.value(&storage, &key), Ok(Some(value)));
        assert_eq!(map.key(&storage, &value), Ok(Some(key.clone())));

        map.remove_value(&mut storage, &value).unwrap();
        map.remove_value(&mut storage, &value).unwrap();

        assert_eq!(map.value(&storage, &key), Ok(None));
        assert_eq!(map.key(&storage, &value), Ok(None));
    }
}
