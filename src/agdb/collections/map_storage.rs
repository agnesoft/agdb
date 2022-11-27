pub mod map_storage_iterator;

mod map_storage_index;
mod map_value_state;

use self::map_storage_index::MapStorageIndex;
use self::map_storage_iterator::MapStorageIterator;
use self::map_value_state::MapValueState;
use super::vec_storage::VecStorage;
use crate::storage::file_storage::FileStorage;
use crate::storage::storage_index::StorageIndex;
use crate::storage::storage_value::StorageValue;
use crate::storage::Storage;
use crate::utilities::stable_hash::StableHash;
use crate::DbError;
use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

#[allow(dead_code)]
pub struct MapStorage<K, T, Data = FileStorage>
where
    K: Default + Eq + Hash + PartialEq + StableHash + StorageValue + Display,
    T: Default + StorageValue,
    Data: Storage,
{
    storage_index: StorageIndex,
    storage: Rc<RefCell<Data>>,
    data_index: MapStorageIndex,
    states: VecStorage<MapValueState, Data>,
    keys: VecStorage<K, Data>,
    values: VecStorage<T, Data>,
}

#[allow(dead_code)]
impl<K, T, Data> MapStorage<K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash + StorageValue + Display,
    T: Default + StorageValue,
    Data: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.states.len()
    }

    pub fn from_storage(storage: Rc<RefCell<Data>>, index: &StorageIndex) -> Result<Self, DbError> {
        let data_index = storage.borrow_mut().value::<MapStorageIndex>(index)?;
        let states = VecStorage::<MapValueState, Data>::from_storage(
            storage.clone(),
            &data_index.states_index,
        )?;
        let keys = VecStorage::<K, Data>::from_storage(storage.clone(), &data_index.keys_index)?;
        let values =
            VecStorage::<T, Data>::from_storage(storage.clone(), &data_index.values_index)?;

        Ok(MapStorage {
            storage_index: *index,
            storage,
            data_index,
            states,
            keys,
            values,
        })
    }

    pub fn insert(&mut self, key: &K, value: &T) -> Result<Option<T>, DbError> {
        self.storage.borrow_mut().transaction();

        let index = self.find_or_free(key)?;
        self.states.set_value(index.0, &MapValueState::Valid)?;
        self.keys.set_value(index.0, key)?;
        self.values.set_value(index.0, value)?;
        self.storage.borrow_mut().commit()?;

        Ok(index.1)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> MapStorageIterator<K, T, Data> {
        MapStorageIterator {
            pos: 0,
            map: self,
            phantom_data: PhantomData,
        }
    }

    pub fn len(&self) -> u64 {
        self.data_index.len
    }

    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        let states = VecStorage::<MapValueState, Data>::new(storage.clone())?;
        let keys = VecStorage::<K, Data>::new(storage.clone())?;
        let values = VecStorage::<T, Data>::new(storage.clone())?;

        let data_index = MapStorageIndex {
            len: 0,
            states_index: states.storage_index(),
            keys_index: keys.storage_index(),
            values_index: values.storage_index(),
        };

        let storage_index = storage.borrow_mut().insert(&data_index)?;

        Ok(MapStorage {
            storage_index,
            storage,
            data_index,
            states,
            keys,
            values,
        })
    }

    pub fn remove(&mut self, key: &K) -> Result<(), DbError> {
        if self.capacity() == 0 {
            return Ok(());
        }

        let mut pos = key.stable_hash() % self.capacity();

        loop {
            match self.states.value(pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid if self.keys.value(pos)? == *key => {
                    self.states.set_value(pos, &MapValueState::Deleted)?;
                    self.set_len(self.len() - 1)?;
                    break;
                }
                MapValueState::Deleted | MapValueState::Valid => pos = self.next_pos(pos),
            }
        }

        if self.len() < self.min_len() {
            self.rehash(self.capacity() / 2)?;
        }

        Ok(())
    }

    pub fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        if self.capacity() < capacity {
            self.rehash(capacity)?;
        }

        Ok(())
    }

    pub fn shrink_to_fit(&mut self) -> Result<(), DbError> {
        self.states.shrink_to_fit()?;
        self.keys.shrink_to_fit()?;
        self.values.shrink_to_fit()
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_index
    }

    pub fn to_hash_map(&self) -> Result<HashMap<K, T>, DbError> {
        let mut map = HashMap::<K, T>::new();
        map.reserve(self.len() as usize);

        for (key, value) in self.iter() {
            map.insert(key, value);
        }

        Ok(map)
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        if self.capacity() == 0 {
            return Ok(None);
        }

        let mut pos = key.stable_hash() % self.capacity();

        loop {
            match self.states.value(pos)? {
                MapValueState::Empty => return Ok(None),
                MapValueState::Valid if self.keys.value(pos)? == *key => {
                    return Ok(Some(self.values.value(pos)?));
                }
                MapValueState::Deleted | MapValueState::Valid => pos = self.next_pos(pos),
            }
        }
    }

    fn find_or_free(&mut self, key: &K) -> Result<(u64, Option<T>), DbError> {
        if self.len() == self.max_len() {
            self.rehash(self.capacity() * 2)?;
        }

        let mut pos = key.stable_hash() % self.capacity();
        let mut old_value: Option<T> = None;

        loop {
            match self.states.value(pos)? {
                MapValueState::Empty | MapValueState::Deleted => {
                    self.set_len(self.len() + 1)?;
                    break;
                }
                MapValueState::Valid if self.keys.value(pos)? == *key => {
                    old_value = Some(self.values.value(pos)?);
                    break;
                }
                MapValueState::Valid => pos = self.next_pos(pos),
            }
        }

        Ok((pos, old_value))
    }

    fn grow(&mut self, current_capacity: u64, new_capacity: u64) -> Result<(), DbError> {
        self.resize(new_capacity)?;
        self.rehash_values(current_capacity, new_capacity)
    }

    fn max_len(&self) -> u64 {
        self.capacity() * 15 / 16
    }

    fn min_len(&self) -> u64 {
        self.capacity() * 7 / 16
    }

    fn next_pos(&self, pos: u64) -> u64 {
        if pos == self.capacity() - 1 {
            0
        } else {
            pos + 1
        }
    }

    fn rehash(&mut self, capacity: u64) -> Result<(), DbError> {
        let current_capacity = self.capacity();
        let new_capacity = max(capacity, 64_u64);

        match current_capacity.cmp(&new_capacity) {
            std::cmp::Ordering::Less => self.grow(current_capacity, new_capacity),
            std::cmp::Ordering::Equal => Ok(()),
            std::cmp::Ordering::Greater => self.shrink(current_capacity, new_capacity),
        }
    }

    fn rehash_empty(&mut self, i: &mut u64) -> Result<(), DbError> {
        *i += 1;
        Ok(())
    }

    fn rehash_deleted(&mut self, i: &mut u64, new_capacity: u64) -> Result<(), DbError> {
        if *i < new_capacity {
            self.states.set_value(*i, &MapValueState::Empty)?;
        }

        *i += 1;
        Ok(())
    }

    fn rehash_value(
        &mut self,
        state: MapValueState,
        i: &mut u64,
        new_capacity: u64,
        empty_list: &mut [bool],
    ) -> Result<(), DbError> {
        match state {
            MapValueState::Empty => self.rehash_empty(i),
            MapValueState::Deleted => self.rehash_deleted(i, new_capacity),
            MapValueState::Valid => self.rehash_valid(i, new_capacity, empty_list),
        }
    }

    fn rehash_valid(
        &mut self,
        i: &mut u64,
        new_capacity: u64,
        empty_list: &mut [bool],
    ) -> Result<(), DbError> {
        let key = self.keys.value(*i)?;
        let mut pos = key.stable_hash() % new_capacity;

        loop {
            if empty_list[pos as usize] {
                empty_list[pos as usize] = false;
                self.swap(*i, pos)?;

                if *i == pos {
                    *i += 1;
                }

                break;
            }

            pos += 1;

            if pos == new_capacity {
                pos = 0;
            }
        }

        Ok(())
    }

    fn rehash_values(&mut self, current_capacity: u64, new_capacity: u64) -> Result<(), DbError> {
        let mut i = 0_u64;
        let mut empty_list = vec![true; new_capacity as usize];

        while i != current_capacity {
            self.rehash_value(self.states.value(i)?, &mut i, new_capacity, &mut empty_list)?;
        }

        Ok(())
    }

    fn resize(&mut self, new_capacity: u64) -> Result<(), DbError> {
        self.states.resize(new_capacity, &MapValueState::Empty)?;
        self.keys.resize(new_capacity, &K::default())?;
        self.values.resize(new_capacity, &T::default())
    }

    fn set_len(&mut self, len: u64) -> Result<(), DbError> {
        self.data_index.len = len;
        self.storage
            .borrow_mut()
            .insert_at(&self.storage_index, 0, &len)
    }

    fn shrink(&mut self, current_capacity: u64, new_capacity: u64) -> Result<(), DbError> {
        self.rehash_values(current_capacity, new_capacity)?;
        self.resize(new_capacity)
    }

    fn swap(&mut self, index: u64, other: u64) -> Result<(), DbError> {
        self.states.swap(index, other)?;
        self.keys.swap(index, other)?;
        self.values.swap(index, other)
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

        assert_eq!(map.to_hash_map(), Ok(expected));
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
    fn insert_reallocate() {
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
    fn insert_reallocate_with_collisions() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MapStorage::<u64, u64>::new(storage).unwrap();

        for i in 0..100 {
            map.insert(&(i * 64), &i).unwrap();
        }

        for i in 0..100 {
            assert_eq!(map.value(&(i * 64)), Ok(Some(i)));
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

        for i in 0..100 {
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
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let map = MapStorage::<u64, u64>::new(storage).unwrap();
        let other = map.to_hash_map().unwrap();

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
