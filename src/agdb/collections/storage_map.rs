use crate::db_error::DbError;
use crate::storage::storage_file::StorageFile;
use crate::storage::storage_index::StorageIndex;
use crate::storage::Storage;
use crate::utilities::serialize::Serialize;
use crate::utilities::stable_hash::StableHash;
use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::size_of;
use std::rc::Rc;

use super::map::map_impl::MapImpl;
use super::map_common::map_data_storage::MapDataStorage;
use super::map_common::map_value::MapValue;
use super::map_common::MapCommon;

pub type StorageMap<K, T, Data = StorageFile> = MapImpl<K, T, MapDataStorage<K, T, Data>>;

#[allow(dead_code)]
impl<K, T, Data> StorageMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: Storage,
{
    pub fn storage_index(&self) -> StorageIndex {
        self.map_common.data.storage_index()
    }
}

impl<K, T, Data> TryFrom<Rc<RefCell<Data>>> for StorageMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(storage: Rc<RefCell<Data>>) -> Result<Self, Self::Error> {
        let storage_index = storage.borrow_mut().insert(&0_u64)?;
        storage.borrow_mut().insert_at(
            &storage_index,
            size_of::<u64>() as u64,
            &vec![MapValue::<K, T>::default()],
        )?;

        Ok(Self {
            map_common: MapCommon::from(MapDataStorage::try_from(storage)?),
            phantom_data: PhantomData,
        })
    }
}

impl<K, T, Data> TryFrom<(Rc<RefCell<Data>>, StorageIndex)> for StorageMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (Rc<RefCell<Data>>, StorageIndex),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            map_common: MapCommon::from(MapDataStorage::try_from(storage_with_index)?),
            phantom_data: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;
    use std::collections::HashMap;

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();

        assert_eq!(map.count(), 3);
        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(15)));
        assert_eq!(map.value(&7), Ok(Some(20)));
    }

    #[test]
    fn insert_reallocate() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

        assert_eq!(map.capacity(), 1);

        for i in 0..100 {
            map.insert(i, i).unwrap();
        }

        assert_eq!(map.count(), 100);
        assert_eq!(map.capacity(), 128);

        for i in 0..100 {
            assert_eq!(map.value(&i), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_reallocate_with_collisions() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

        for i in 0..100 {
            map.insert(i * 64, i).unwrap();
        }

        for i in 0..100 {
            assert_eq!(map.value(&(i * 64)), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_same_key() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

        assert_eq!(map.insert(1, 10), Ok(None));
        assert_eq!(map.insert(5, 15), Ok(None));
        assert_eq!(map.count(), 2);
        assert_eq!(map.insert(5, 20), Ok(Some(15)));
        assert_eq!(map.count(), 2);

        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(20)));
    }

    #[test]
    fn iter() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();
        map.insert(2, 30).unwrap();
        map.insert(4, 13).unwrap();
        map.remove(&7).unwrap();

        let mut actual = map.iter().collect::<Vec<(i64, i64)>>();
        actual.sort();
        let expected: Vec<(i64, i64)> = vec![(1, 10), (2, 30), (4, 13), (5, 15)];

        assert_eq!(actual, expected);
    }

    #[test]
    fn remove() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();

        assert_eq!(map.count(), 3);
        map.remove(&5).unwrap();

        assert_eq!(map.count(), 2);
        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(None));
        assert_eq!(map.value(&7), Ok(Some(20)));
    }

    #[test]
    fn remove_deleted() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();

        assert_eq!(map.count(), 3);

        map.remove(&5).unwrap();

        assert_eq!(map.count(), 2);
        assert_eq!(map.value(&5), Ok(None));

        map.remove(&5).unwrap();

        assert_eq!(map.count(), 2);
    }

    #[test]
    fn remove_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

        assert_eq!(map.count(), 0);
        assert_eq!(map.remove(&0), Ok(()));
        assert_eq!(map.count(), 0);
    }

    #[test]
    fn remove_shrinks_capacity() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

        for i in 0..100 {
            map.insert(i, i).unwrap();
        }

        assert_eq!(map.count(), 100);
        assert_eq!(map.capacity(), 128);

        for i in 0..100 {
            map.remove(&i).unwrap();
        }

        assert_eq!(map.count(), 0);
        assert_eq!(map.capacity(), 64);
    }

    #[test]
    fn reserve_larger() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();
        map.insert(1, 1).unwrap();

        let capacity = map.capacity() + 10;
        let size = map.count();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.count(), size);
        assert_eq!(map.value(&1), Ok(Some(1)));
    }

    #[test]
    fn reserve_same() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();
        map.insert(1, 1).unwrap();

        let capacity = map.capacity();
        let size = map.count();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.count(), size);
    }

    #[test]
    fn reserve_smaller() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();
        map.insert(1, 1).unwrap();

        let current_capacity = map.capacity();
        let capacity = current_capacity - 10;
        let size = map.count();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), current_capacity);
        assert_eq!(map.count(), size);
    }

    #[test]
    fn to_hash_map() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();
        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();
        map.remove(&5).unwrap();

        let other = map.to_hash_map().unwrap();

        assert_eq!(other.len(), 2);
        assert_eq!(other.get(&1), Some(&10));
        assert_eq!(other.get(&5), None);
        assert_eq!(other.get(&7), Some(&20));
    }

    #[test]
    fn to_hash_map_empty() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let map = StorageMap::<i64, i64>::try_from(storage).unwrap();
        let other = map.to_hash_map().unwrap();

        assert_eq!(other.len(), 0);
    }

    #[test]
    fn try_from_storage_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let index;

        {
            let mut map = StorageMap::<i64, i64>::try_from(storage.clone()).unwrap();
            map.insert(1, 1).unwrap();
            map.insert(3, 2).unwrap();
            map.insert(5, 3).unwrap();
            map.remove(&3).unwrap();
            index = map.storage_index();
        }

        let map = StorageMap::<i64, i64>::try_from((storage, index)).unwrap();

        let mut expected = HashMap::<i64, i64>::new();
        expected.insert(1, 1);
        expected.insert(5, 3);

        assert_eq!(map.to_hash_map(), Ok(expected));
    }

    #[test]
    fn try_from_storage_missing_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        assert_eq!(
            StorageMap::<i64, i64>::try_from((storage, StorageIndex::from(1_i64)))
                .err()
                .unwrap(),
            DbError::from("index '1' not found")
        );
    }

    #[test]
    fn value_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let map = StorageMap::<i64, i64>::try_from(storage).unwrap();

        assert_eq!(map.value(&0), Ok(None));
    }

    #[test]
    fn values_at_end() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(127, 10).unwrap();
        map.insert(255, 11).unwrap();
        map.insert(191, 12).unwrap();

        assert_eq!(map.value(&127), Ok(Some(10)));
        assert_eq!(map.value(&255), Ok(Some(11)));
        assert_eq!(map.value(&191), Ok(Some(12)));
    }
}
