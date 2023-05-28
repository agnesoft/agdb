use super::map::map_data_storage::MapDataStorage;
use super::map::multi_map_impl::MultiMapImpl;
use super::multi_map::MultiMap;
use crate::collections::vec::VecValue;
use crate::db::db_error::DbError;
use crate::storage::file_storage::FileStorage;
use crate::storage::storage_index::StorageIndex;
use crate::storage::Storage;
use crate::utilities::stable_hash::StableHash;
use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

pub type MultiMapStorage<K, T, Data = FileStorage> = MultiMapImpl<K, T, MapDataStorage<K, T, Data>>;

impl<K, T, Data> MultiMapStorage<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + VecValue,
    T: Clone + Default + Eq + PartialEq + VecValue,
    Data: Storage,
{
    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        Ok(Self {
            data: MapDataStorage::<K, T, Data>::new(storage)?,
            phantom_marker: PhantomData,
        })
    }

    pub fn from_storage(storage: Rc<RefCell<Data>>, index: StorageIndex) -> Result<Self, DbError> {
        Ok(Self {
            data: MapDataStorage::<K, T, Data>::from_storage(storage, index)?,
            phantom_marker: PhantomData,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.data.storage_index()
    }

    pub fn to_multi_map(&self) -> Result<MultiMap<K, T>, DbError> {
        Ok(MultiMap {
            data: self.data.to_map_data_memory()?,
            phantom_marker: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn new() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = MultiMapStorage::<u64, String>::new(storage).unwrap();
        map.insert(&1, &"Hello".to_string()).unwrap();
        map.insert(&1, &"World".to_string()).unwrap();
        map.insert(&1, &"!".to_string()).unwrap();

        let mut values = Vec::<(u64, String)>::new();
        values.reserve(3);

        for (key, value) in map.iter() {
            values.push((key, value));
        }

        values.sort();

        assert_eq!(
            values,
            vec![
                (1, "!".to_string()),
                (1, "Hello".to_string()),
                (1, "World".to_string())
            ]
        );
    }

    #[test]
    fn from_storage() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let storage_index;

        {
            let mut map = MultiMapStorage::<u64, String>::new(storage.clone()).unwrap();
            map.insert(&1, &"Hello".to_string()).unwrap();
            map.insert(&1, &"World".to_string()).unwrap();
            map.insert(&1, &"!".to_string()).unwrap();
            storage_index = map.storage_index();
        }

        let map = MultiMapStorage::<u64, String>::from_storage(storage, storage_index).unwrap();

        let mut values = Vec::<(u64, String)>::new();
        values.reserve(3);

        for (key, value) in map.iter() {
            values.push((key, value));
        }

        values.sort();

        assert_eq!(
            values,
            vec![
                (1, "!".to_string()),
                (1, "Hello".to_string()),
                (1, "World".to_string())
            ]
        );
    }

    #[test]
    fn to_multi_map() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MultiMapStorage::<u64, String>::new(storage).unwrap();
        map.insert(&1, &"Hello".to_string()).unwrap();
        map.insert(&1, &"World".to_string()).unwrap();
        map.insert(&1, &"!".to_string()).unwrap();

        let mem_map = map.to_multi_map().unwrap();

        let mut values = Vec::<(u64, String)>::new();
        values.reserve(3);

        for (key, value) in mem_map.iter() {
            values.push((key, value));
        }

        values.sort();

        assert_eq!(
            values,
            vec![
                (1, "!".to_string()),
                (1, "Hello".to_string()),
                (1, "World".to_string())
            ]
        );
    }
}
