use super::map_common::map_data_memory::MapDataMemory;
use super::map_common::map_data_storage::MapDataStorage;
use super::map_common::MapCommon;
use super::multi_map::multi_map_impl::MultiMapImpl;
use super::multi_map::MultiMap;
use crate::db_error::DbError;
use crate::storage::storage_file::StorageFile;
use crate::storage::storage_index::StorageIndex;
use crate::storage::Storage;
use crate::utilities::serialize::Serialize;
use crate::utilities::stable_hash::StableHash;
use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

pub type StorageMultiMap<K, T, Data = StorageFile> = MultiMapImpl<K, T, MapDataStorage<K, T, Data>>;

impl<K, T, Data> StorageMultiMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
    Data: Storage,
{
    pub fn storage_index(&self) -> StorageIndex {
        self.map_common.data.storage_index()
    }

    pub fn to_multi_map(&self) -> Result<MultiMap<K, T>, DbError> {
        Ok(MultiMap::<K, T> {
            map_common: MapCommon::from(MapDataMemory::<K, T> {
                data: self.map_common.data.values()?,
                count: self.map_common.data.count(),
            }),
        })
    }
}

impl<K, T, Data> TryFrom<Rc<RefCell<Data>>> for StorageMultiMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(storage: Rc<RefCell<Data>>) -> Result<Self, Self::Error> {
        Ok(Self {
            map_common: MapCommon::from(MapDataStorage::try_from(storage)?),
        })
    }
}

impl<K, T, Data> TryFrom<(Rc<RefCell<Data>>, StorageIndex)> for StorageMultiMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (Rc<RefCell<Data>>, StorageIndex),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            map_common: MapCommon::from(MapDataStorage::try_from(storage_with_index)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

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
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

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
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        for i in 0..50 {
            map.insert(i * 64, i).unwrap();
            map.insert(i * 64, i + 1).unwrap();
        }

        for i in 0..50 {
            assert_eq!(map.value(&(i * 64)), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_same_key() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        assert_eq!(map.count(), 2);
        map.insert(5, 20).unwrap();
        assert_eq!(map.count(), 3);

        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(15)));
    }

    #[test]
    fn iter() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();
        map.insert(2, 30).unwrap();
        map.insert(2, 50).unwrap();
        map.insert(4, 13).unwrap();
        map.remove_key(&7).unwrap();

        let mut actual = map.iter().collect::<Vec<(i64, i64)>>();
        actual.sort();
        let expected: Vec<(i64, i64)> = vec![(1, 10), (2, 30), (2, 50), (4, 13), (5, 15), (5, 15)];

        assert_eq!(actual, expected);
    }

    #[test]
    fn remove_deleted_key() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();

        assert_eq!(map.count(), 3);

        map.remove_key(&5).unwrap();

        assert_eq!(map.count(), 2);
        assert_eq!(map.value(&5), Ok(None));

        map.remove_key(&5).unwrap();

        assert_eq!(map.count(), 2);
    }

    #[test]
    fn remove_key() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 7).unwrap();
        map.insert(5, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(5, 20).unwrap();

        assert_eq!(map.count(), 4);
        map.remove_key(&5).unwrap();

        assert_eq!(map.count(), 1);
        assert_eq!(map.value(&1), Ok(Some(7)));
        assert_eq!(map.values(&5), Ok(Vec::<i64>::new()));
    }

    #[test]
    fn remove_missing_key() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        assert_eq!(map.count(), 0);

        map.remove_key(&5).unwrap();

        assert_eq!(map.count(), 0);
    }

    #[test]
    fn remove_shrinks_capacity() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        for i in 0..100 {
            map.insert(i, i).unwrap();
        }

        assert_eq!(map.count(), 100);
        assert_eq!(map.capacity(), 128);

        for i in 0..100 {
            map.remove_key(&i).unwrap();
        }

        assert_eq!(map.count(), 0);
        assert_eq!(map.capacity(), 64);
    }

    #[test]
    fn remove_value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 7).unwrap();
        map.insert(5, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(5, 20).unwrap();

        assert_eq!(map.count(), 4);
        map.remove_value(&5, &15).unwrap();

        assert_eq!(map.count(), 3);
        assert_eq!(map.value(&1), Ok(Some(7)));
        assert_eq!(map.values(&5), Ok(vec![10_i64, 20_i64]));
    }

    #[test]
    fn remove_missing_value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        map.remove_value(&5, &10).unwrap();

        assert_eq!(map.count(), 0);
    }

    #[test]
    fn reserve_larger() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

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
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

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
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 1).unwrap();

        let current_capacity = map.capacity();
        let capacity = current_capacity - 10;
        let size = map.count();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), current_capacity);
        assert_eq!(map.count(), size);
    }

    #[test]
    fn to_multi_map() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();
        map.remove_key(&5).unwrap();

        let other = map.to_multi_map().unwrap();

        assert_eq!(other.count(), 2);
        assert_eq!(other.value(&1).unwrap(), Some(10));
        assert_eq!(other.value(&5).unwrap(), None);
        assert_eq!(other.value(&7).unwrap(), Some(20));
    }

    #[test]
    fn to_multi_map_empty() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();
        let other = map.to_multi_map().unwrap();

        assert_eq!(other.count(), 0);
    }

    #[test]
    fn try_from_storage_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let index;

        {
            let mut map = StorageMultiMap::<i64, i64>::try_from(storage.clone()).unwrap();
            map.insert(1, 1).unwrap();
            map.insert(3, 2).unwrap();
            map.insert(3, 3).unwrap();
            map.insert(5, 3).unwrap();
            map.remove_key(&1).unwrap();
            index = map.storage_index();
        }

        let map = StorageMultiMap::<i64, i64>::try_from((storage, index)).unwrap();

        assert_eq!(
            map.iter().collect::<Vec<(i64, i64)>>(),
            vec![(3_i64, 2_i64), (3_i64, 3_i64), (5_i64, 3_i64)]
        );
    }

    #[test]
    fn try_from_storage_missing_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));

        assert_eq!(
            StorageMultiMap::<i64, i64>::try_from((storage, StorageIndex::from(1_i64)))
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
        let map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        assert_eq!(map.value(&0), Ok(None));
    }

    #[test]
    fn values_at_end() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            StorageFile::try_from(test_file.file_name().clone()).unwrap(),
        ));
        let mut map = StorageMultiMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(127, 10).unwrap();
        map.insert(255, 11).unwrap();
        map.insert(191, 12).unwrap();

        assert_eq!(map.value(&127), Ok(Some(10)));
        assert_eq!(map.value(&255), Ok(Some(11)));
        assert_eq!(map.value(&191), Ok(Some(12)));
    }
}
