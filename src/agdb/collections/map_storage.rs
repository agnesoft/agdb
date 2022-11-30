use super::map::map_data_storage::MapDataStorage;
use super::map::map_impl::MapImpl;
use super::map::multi_map_impl::MultiMapImpl;
use crate::storage::file_storage::FileStorage;
use crate::storage::storage_index::StorageIndex;
use crate::storage::storage_value::StorageValue;
use crate::storage::Storage;
use crate::utilities::stable_hash::StableHash;
use crate::DbError;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

pub type MapStorage<K, T, Data = FileStorage> = MapImpl<K, T, MapDataStorage<K, T, Data>>;

impl<K, T, Data> MapStorage<K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash + StorageValue,
    T: Default + Eq + PartialEq + StorageValue,
    Data: Storage,
{
    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        Ok(Self {
            multi_map: MultiMapImpl::<K, T, MapDataStorage<K, T, Data>> {
                data: MapDataStorage::<K, T, Data>::new(storage)?,
                phantom_marker: PhantomData,
            },
        })
    }

    pub fn from_storage(storage: Rc<RefCell<Data>>, index: &StorageIndex) -> Result<Self, DbError> {
        Ok(Self {
            multi_map: MultiMapImpl::<K, T, MapDataStorage<K, T, Data>> {
                data: MapDataStorage::<K, T, Data>::from_storage(storage, index)?,
                phantom_marker: PhantomData,
            },
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.multi_map.data.storage_index()
    }

    pub fn to_hash_map(&self) -> HashMap<K, T> {
        let mut map = HashMap::<K, T>::new();
        map.reserve(self.len() as usize);

        for (key, value) in self.iter() {
            map.insert(key, value);
        }

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;
    use std::collections::HashMap;

    #[test]
    fn from_storage_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let index;

        {
            let mut map = MapStorage::<u64, u64>::new(storage.clone()).unwrap();
            map.insert(&1, &1).unwrap();
            map.insert(&3, &2).unwrap();
            map.insert(&5, &3).unwrap();
            map.remove(&3).unwrap();
            index = map.storage_index();
        }

        let map = MapStorage::<u64, u64>::from_storage(storage, &index).unwrap();

        let mut expected = HashMap::<u64, u64>::new();
        expected.insert(1, 1);
        expected.insert(5, 3);

        assert_eq!(map.to_hash_map(), expected);
    }

    #[test]
    fn from_storage_missing_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        assert_eq!(
            MapStorage::<u64, u64>::from_storage(storage, &StorageIndex::from(1_u64))
                .err()
                .unwrap(),
            DbError::from("FileStorage error: index (1) not found")
        );
    }

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();

        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();

        assert_eq!(map.len(), 3);
        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(15)));
        assert_eq!(map.value(&7), Ok(Some(20)));
    }

    #[test]
    fn insert_reallocates() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();

        assert_eq!(map.capacity(), 0);

        for i in 0..100 {
            map.insert(&i, &i).unwrap();
        }

        assert_eq!(map.len(), 100);
        assert_eq!(map.capacity(), 128);

        for i in 0..100 {
            assert_eq!(map.value(&i), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_reallocates_with_collisions() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();

        for i in 1..100 {
            map.insert(&(i * 64 - 1), &i).unwrap();
        }

        for i in 1..100 {
            assert_eq!(map.value(&(i * 64 - 1)), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_same_key() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();

        assert_eq!(map.insert(&1, &10), Ok(None));
        assert_eq!(map.insert(&5, &15), Ok(None));
        assert_eq!(map.len(), 2);
        assert_eq!(map.insert(&5, &20), Ok(Some(15)));
        assert_eq!(map.len(), 2);

        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(20)));
    }

    #[test]
    fn is_empty() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();

        assert!(map.is_empty());
        map.insert(&1, &10).unwrap();
        assert!(!map.is_empty());
        map.remove(&1).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn iter() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();

        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();
        map.insert(&2, &30).unwrap();
        map.insert(&4, &13).unwrap();
        map.remove(&7).unwrap();

        let mut actual = map.iter().collect::<Vec<(u64, u64)>>();
        actual.sort();
        let expected: Vec<(u64, u64)> = vec![(1, 10), (2, 30), (4, 13), (5, 15)];

        assert_eq!(actual, expected);
    }

    #[test]
    fn remove() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();

        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();

        assert_eq!(map.len(), 3);
        map.remove(&5).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(None));
        assert_eq!(map.value(&7), Ok(Some(20)));
    }

    #[test]
    fn remove_deleted() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();

        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();

        assert_eq!(map.len(), 3);

        map.remove(&5).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(map.value(&5), Ok(None));

        map.remove(&5).unwrap();

        assert_eq!(map.len(), 2);
    }

    #[test]
    fn remove_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();

        assert_eq!(map.len(), 0);
        assert_eq!(map.remove(&0), Ok(()));
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn remove_shrinks_capacity() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();

        for i in 0..100 {
            map.insert(&i, &i).unwrap();
        }

        assert_eq!(map.len(), 100);
        assert_eq!(map.capacity(), 128);

        for i in (0..100).rev() {
            map.remove(&i).unwrap();
        }

        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), 64);
    }

    #[test]
    fn reserve_larger() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();
        map.insert(&1, &1).unwrap();

        let capacity = map.capacity() + 10;
        let size = map.len();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.len(), size);
        assert_eq!(map.value(&1), Ok(Some(1)));
    }

    #[test]
    fn reserve_same() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();
        map.insert(&1, &1).unwrap();

        let capacity = map.capacity();
        let size = map.len();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.len(), size);
    }

    #[test]
    fn reserve_smaller() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();
        map.insert(&1, &1).unwrap();

        let current_capacity = map.capacity();
        let capacity = current_capacity - 10;
        let size = map.len();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), current_capacity);
        assert_eq!(map.len(), size);
    }

    #[test]
    fn to_hash_map() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();
        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();
        map.remove(&5).unwrap();

        let other = map.to_hash_map();

        assert_eq!(other.len(), 2);
        assert_eq!(other.get(&1), Some(&10));
        assert_eq!(other.get(&5), None);
        assert_eq!(other.get(&7), Some(&20));
    }

    #[test]
    fn to_hash_map_empty() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let map = MapStorage::<u64, u64>::new(storage).unwrap();
        let other = map.to_hash_map();

        assert_eq!(other.len(), 0);
    }

    #[test]
    fn value_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let map = MapStorage::<u64, u64>::new(storage).unwrap();

        assert_eq!(map.value(&0), Ok(None));
    }

    #[test]
    fn values_at_end() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();

        map.insert(&127, &10).unwrap();
        map.insert(&255, &11).unwrap();
        map.insert(&191, &12).unwrap();

        assert_eq!(map.value(&127), Ok(Some(10)));
        assert_eq!(map.value(&255), Ok(Some(11)));
        assert_eq!(map.value(&191), Ok(Some(12)));
    }
}
