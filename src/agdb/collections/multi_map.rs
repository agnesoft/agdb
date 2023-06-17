use super::map::DbMapData;
use super::map::MapData;
use super::map::MapIterator;
use super::map::MapValueState;
use crate::collections::vec::VecValue;
use crate::db::db_error::DbError;
use crate::storage::file_storage::FileStorage;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::stable_hash::StableHash;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

pub struct MultiMapImpl<K, T, Data>
where
    Data: MapData<K, T>,
{
    pub(crate) data: Data,
    pub(crate) phantom_marker: PhantomData<(K, T)>,
}

pub struct MultiMapIterator<'a, K, T, Data>
where
    Data: MapData<K, T>,
{
    pub pos: u64,
    pub key: &'a K,
    pub data: &'a Data,
    pub phantom_data: PhantomData<T>,
}

impl<'a, K, T, Data> Iterator for MultiMapIterator<'a, K, T, Data>
where
    K: Default + PartialEq,
    T: Default,
    Data: MapData<K, T>,
{
    type Item = (K, T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current_pos = self.pos;
            self.pos = if self.data.capacity() == 0 || self.pos == self.data.capacity() - 1 {
                0
            } else {
                self.pos + 1
            };

            match self.data.state(current_pos).unwrap_or_default() {
                MapValueState::Empty => break,
                MapValueState::Deleted => {}
                MapValueState::Valid => {
                    let key = self.data.key(current_pos).unwrap_or_default();

                    if key == *self.key {
                        let value = self.data.value(current_pos).unwrap_or_default();
                        return Some((key, value));
                    }
                }
            }
        }

        None
    }
}

impl<K, T, Data> MultiMapImpl<K, T, Data>
where
    K: Default + PartialEq + StableHash,
    T: Default + PartialEq,
    Data: MapData<K, T>,
{
    pub fn capacity(&self) -> u64 {
        self.data.capacity()
    }

    pub fn contains(&self, key: &K) -> Result<bool, DbError> {
        if self.capacity() == 0 {
            return Ok(false);
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            match self.data.state(pos)? {
                MapValueState::Empty => return Ok(false),
                MapValueState::Valid if self.data.key(pos)? == *key => return Ok(true),
                MapValueState::Valid | MapValueState::Deleted => pos = self.next_pos(pos),
            }
        }
    }

    pub fn contains_value(&self, key: &K, value: &T) -> Result<bool, DbError> {
        if self.capacity() == 0 {
            return Ok(false);
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            match self.data.state(pos)? {
                MapValueState::Empty => return Ok(false),
                MapValueState::Valid
                    if self.data.key(pos)? == *key && self.data.value(pos)? == *value =>
                {
                    return Ok(true)
                }
                MapValueState::Valid | MapValueState::Deleted => pos = self.next_pos(pos),
            }
        }
    }

    pub fn insert(&mut self, key: &K, value: &T) -> Result<(), DbError> {
        let id = self.data.transaction();
        let index = self.free_index(key)?;
        self.data.set_state(index, MapValueState::Valid)?;
        self.data.set_key(index, key)?;
        self.data.set_value(index, value)?;
        self.data.set_len(self.len() + 1)?;
        self.data.commit(id)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> MapIterator<K, T, Data> {
        MapIterator {
            pos: 0,
            data: &self.data,
            phantom_data: PhantomData,
        }
    }

    pub fn iter_key<'a>(&'a self, key: &'a K) -> MultiMapIterator<K, T, Data> {
        let pos = if self.capacity() == 0 {
            0
        } else {
            key.stable_hash() % self.capacity()
        };

        MultiMapIterator {
            pos,
            key,
            data: &self.data,
            phantom_data: PhantomData,
        }
    }

    pub fn len(&self) -> u64 {
        self.data.len()
    }

    pub fn remove_key(&mut self, key: &K) -> Result<(), DbError> {
        if self.capacity() == 0 {
            return Ok(());
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let mut len = self.len();

        let id = self.data.transaction();

        loop {
            match self.data.state(pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid if self.data.key(pos)? == *key => {
                    self.drop_value(pos)?;
                    len -= 1;
                }
                MapValueState::Valid | MapValueState::Deleted => {}
            }

            pos = self.next_pos(pos);
        }

        if len != self.len() {
            self.data.set_len(len)?;

            if self.len() <= self.min_len() {
                self.rehash(self.capacity() / 2)?;
            }
        }

        self.data.commit(id)
    }

    pub fn remove_value(&mut self, key: &K, value: &T) -> Result<(), DbError> {
        if self.capacity() == 0 {
            return Ok(());
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        let id = self.data.transaction();

        loop {
            match self.data.state(pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid
                    if self.data.key(pos)? == *key && self.data.value(pos)? == *value =>
                {
                    self.remove_index(pos)?;
                    break;
                }
                MapValueState::Valid | MapValueState::Deleted => pos = self.next_pos(pos),
            }
        }

        self.data.commit(id)
    }

    pub fn replace(&mut self, key: &K, value: &T, new_value: &T) -> Result<(), DbError> {
        if self.capacity() == 0 {
            return Ok(());
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        let id = self.data.transaction();

        loop {
            match self.data.state(pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid
                    if self.data.key(pos)? == *key && self.data.value(pos)? == *value =>
                {
                    self.data.set_value(pos, new_value)?;
                    break;
                }
                MapValueState::Valid | MapValueState::Deleted => pos = self.next_pos(pos),
            }
        }

        self.data.commit(id)
    }

    pub fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        if self.capacity() < capacity {
            self.rehash(capacity)?;
        }

        Ok(())
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        if self.capacity() == 0 {
            return Ok(None);
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            match self.data.state(pos)? {
                MapValueState::Empty => {
                    return Ok(None);
                }
                MapValueState::Valid if self.data.key(pos)? == *key => {
                    return Ok(Some(self.data.value(pos)?));
                }
                MapValueState::Valid | MapValueState::Deleted => pos = self.next_pos(pos),
            }
        }
    }

    pub fn values(&self, key: &K) -> Result<Vec<T>, DbError> {
        if self.capacity() == 0 {
            return Ok(vec![]);
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let mut values = Vec::<T>::new();

        loop {
            match self.data.state(pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid if self.data.key(pos)? == *key => {
                    values.push(self.data.value(pos)?)
                }
                MapValueState::Valid | MapValueState::Deleted => {}
            }

            pos = self.next_pos(pos)
        }

        Ok(values)
    }

    pub fn values_count(&self, key: &K) -> Result<u64, DbError> {
        if self.capacity() == 0 {
            return Ok(0);
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let mut result = 0;

        loop {
            match self.data.state(pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid if self.data.key(pos)? == *key => {
                    result += 1;
                }
                MapValueState::Valid | MapValueState::Deleted => {}
            }

            pos = self.next_pos(pos)
        }

        Ok(result)
    }

    fn drop_value(&mut self, pos: u64) -> Result<(), DbError> {
        self.data.set_state(pos, MapValueState::Deleted)?;
        self.data.set_key(pos, &K::default())?;
        self.data.set_value(pos, &T::default())
    }

    fn free_index(&mut self, key: &K) -> Result<u64, DbError> {
        if self.len() >= self.max_len() {
            self.rehash(self.capacity() * 2)?;
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            match self.data.state(pos)? {
                MapValueState::Empty | MapValueState::Deleted => break,
                MapValueState::Valid => pos = self.next_pos(pos),
            }
        }

        Ok(pos)
    }

    fn grow(&mut self, current_capacity: u64, new_capacity: u64) -> Result<(), DbError> {
        self.data.resize(new_capacity)?;
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
        let new_capacity = std::cmp::max(capacity, 64_u64);

        match current_capacity.cmp(&new_capacity) {
            std::cmp::Ordering::Less => self.grow(current_capacity, new_capacity),
            std::cmp::Ordering::Equal => Ok(()),
            std::cmp::Ordering::Greater => self.shrink(current_capacity, new_capacity),
        }
    }

    fn rehash_deleted(&mut self, i: &mut u64, new_capacity: u64) -> Result<(), DbError> {
        if *i < new_capacity {
            self.data.set_state(*i, MapValueState::Empty)?;
        }

        *i += 1;
        Ok(())
    }

    fn rehash_empty(&mut self, i: &mut u64) -> Result<(), DbError> {
        *i += 1;
        Ok(())
    }

    fn rehash_valid(
        &mut self,
        i: &mut u64,
        new_capacity: u64,
        empty_list: &mut [bool],
    ) -> Result<(), DbError> {
        let key = self.data.key(*i)?;
        let mut pos = key.stable_hash() % new_capacity;

        loop {
            if empty_list[pos as usize] {
                empty_list[pos as usize] = false;
                self.data.swap(*i, pos)?;

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

    fn rehash_values(&mut self, current_capacity: u64, new_capacity: u64) -> Result<(), DbError> {
        let mut i = 0_u64;
        let mut empty_list = vec![true; new_capacity as usize];

        while i != current_capacity {
            self.rehash_value(self.data.state(i)?, &mut i, new_capacity, &mut empty_list)?;
        }

        Ok(())
    }

    fn remove_index(&mut self, index: u64) -> Result<(), DbError> {
        self.drop_value(index)?;
        self.data.set_len(self.len() - 1)?;

        if self.len() <= self.min_len() {
            self.rehash(self.capacity() / 2)?;
        }

        Ok(())
    }

    fn shrink(&mut self, current_capacity: u64, new_capacity: u64) -> Result<(), DbError> {
        self.rehash_values(current_capacity, new_capacity)?;
        self.data.resize(new_capacity)
    }
}

pub type MultiMapStorage<K, T, Data = FileStorage> = MultiMapImpl<K, T, DbMapData<K, T, Data>>;

impl<K, T, Data> MultiMapStorage<K, T, Data>
where
    K: Clone + Default + PartialEq + VecValue,
    T: Clone + Default + PartialEq + VecValue,
    Data: Storage,
{
    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        Ok(Self {
            data: DbMapData::<K, T, Data>::new(storage)?,
            phantom_marker: PhantomData,
        })
    }

    pub fn from_storage(storage: Rc<RefCell<Data>>, index: StorageIndex) -> Result<Self, DbError> {
        Ok(Self {
            data: DbMapData::<K, T, Data>::from_storage(storage, index)?,
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
    fn iter_key() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = MultiMapStorage::<u64, u64>::new(storage).unwrap();

        assert_eq!(map.iter_key(&1).count(), 0);

        map.insert(&3, &30).unwrap();
        map.insert(&1, &10).unwrap();
        map.insert(&1, &20).unwrap();
        map.insert(&1, &30).unwrap();
        map.insert(&4, &40).unwrap();
        map.remove_value(&1, &10).unwrap();

        let value = map.iter_key(&1).find(|v| v.1 == 20).unwrap();
        assert_eq!(value, (1, 20));

        let mut values = Vec::<(u64, u64)>::new();
        values.reserve(2);

        for (key, value) in map.iter_key(&1) {
            values.push((key, value));
        }

        values.sort();

        assert_eq!(values, vec![(1, 20), (1, 30)]);
    }

    #[test]
    fn remove_value_empty_map() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = MultiMapStorage::<u64, String>::new(storage).unwrap();

        assert!(map.remove_value(&10, &"Hello".to_string()).is_ok());
    }

    #[test]
    fn remove_missing_value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = MultiMapStorage::<u64, String>::new(storage).unwrap();
        map.insert(&11, &"Hello".to_string()).unwrap();

        assert!(map.remove_value(&10, &"Hello".to_string()).is_ok());
    }

    #[test]
    fn remove_value_shrinks_capacity() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = MultiMapStorage::<u64, u64>::new(storage).unwrap();

        for i in 0..100 {
            map.insert(&i, &i).unwrap();
        }

        assert_eq!(map.len(), 100);
        assert_eq!(map.capacity(), 128);

        for i in (0..100).rev() {
            map.remove_value(&i, &i).unwrap();
        }

        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), 64);
    }

    #[test]
    fn replace_empty_map() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = MultiMapStorage::<u64, String>::new(storage).unwrap();

        assert!(map
            .replace(&10, &"Hello".to_string(), &"World".to_string())
            .is_ok());
    }

    #[test]
    fn replace_missing_value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = MultiMapStorage::<u64, String>::new(storage).unwrap();
        map.insert(&11, &"Hello".to_string()).unwrap();

        assert!(map
            .replace(&10, &"Hello".to_string(), &"World".to_string())
            .is_ok());
    }

    #[test]
    fn replace_deleted() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = MultiMapStorage::<u64, String>::new(storage).unwrap();
        map.insert(&10, &"Hello".to_string()).unwrap();
        map.remove_key(&10).unwrap();

        assert!(map
            .replace(&10, &"Hello".to_string(), &"World".to_string())
            .is_ok());
    }

    #[test]
    fn values_count() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = MultiMapStorage::<u64, String>::new(storage).unwrap();

        assert_eq!(map.values_count(&4), Ok(0));

        map.insert(&1, &"Hello".to_string()).unwrap();
        map.insert(&1, &"World".to_string()).unwrap();
        map.insert(&1, &"!".to_string()).unwrap();
        map.insert(&2, &"a".to_string()).unwrap();
        map.insert(&3, &"b".to_string()).unwrap();
        map.remove_value(&1, &"World".to_string()).unwrap();

        assert_eq!(map.values_count(&1), Ok(2));
        assert_eq!(map.values_count(&2), Ok(1));
        assert_eq!(map.values_count(&3), Ok(1));
        assert_eq!(map.values_count(&4), Ok(0));
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
}
