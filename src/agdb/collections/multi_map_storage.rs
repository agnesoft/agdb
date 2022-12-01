use super::map::map_data_storage::MapDataStorage;
use super::map::multi_map_impl::MultiMapImpl;
use crate::storage::file_storage::FileStorage;
use crate::storage::storage_index::StorageIndex;
use crate::storage::storage_value::StorageValue;
use crate::storage::Storage;
use crate::utilities::stable_hash::StableHash;
use crate::DbError;
use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

pub type MultiMapStorage<K, T, Data = FileStorage> = MultiMapImpl<K, T, MapDataStorage<K, T, Data>>;

#[allow(dead_code)]
impl<K, T, Data> MultiMapStorage<K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash + StorageValue,
    T: Default + Eq + PartialEq + StorageValue,
    Data: Storage,
{
    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        Ok(Self {
            data: MapDataStorage::<K, T, Data>::new(storage)?,
            phantom_marker: PhantomData,
        })
    }

    pub fn from_storage(storage: Rc<RefCell<Data>>, index: &StorageIndex) -> Result<Self, DbError> {
        Ok(Self {
            data: MapDataStorage::<K, T, Data>::from_storage(storage, index)?,
            phantom_marker: PhantomData,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.data.storage_index()
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

        let index;

        {
            let mut map = MultiMapStorage::<u64, String>::new(storage.clone()).unwrap();
            map.insert(&1, &"Hello".to_string()).unwrap();
            map.insert(&1, &"World".to_string()).unwrap();
            map.insert(&1, &"!".to_string()).unwrap();
            index = map.storage_index();
        }

        let map = MultiMapStorage::<u64, String>::from_storage(storage, &index).unwrap();

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
}
