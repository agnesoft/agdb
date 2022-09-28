use super::file_storage_data::FileStorageData;
use super::serialize::Serialize;
use super::stable_hash::StableHash;
use super::storage_data::StorageData;
use super::storage_hash_map_data::StorageHashMapData;
use super::storage_hash_map_key_value::StorageHashMapKeyValue;
use super::storage_hash_map_meta_value::MetaValue;
use super::Storage;
use crate::DbError;
use std::collections::HashMap;
use std::hash::Hash;

#[allow(dead_code)]
pub(crate) struct StorageHashMap<K, T, Data = FileStorageData>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: StorageData,
{
    storage: std::rc::Rc<std::cell::RefCell<Storage<Data>>>,
    storage_index: i64,
    size: u64,
    capacity: u64,
    phantom_data: std::marker::PhantomData<(K, T)>,
}

#[allow(dead_code)]
impl<K, T, Data> StorageHashMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: StorageData,
{
    pub(crate) fn capacity(&self) -> u64 {
        self.capacity
    }

    pub(crate) fn insert(&mut self, key: K, value: T) -> Result<Option<T>, DbError> {
        self.storage.borrow_mut().transaction();
        let free = self.find_or_free(&key)?;
        self.insert_value(free.0, key, value)?;

        if free.1.meta_value == MetaValue::Valid {
            self.storage.borrow_mut().commit()?;
            Ok(Some(free.1.value))
        } else {
            self.set_size(self.size + 1)?;
            self.storage.borrow_mut().commit()?;
            Ok(None)
        }
    }

    pub(crate) fn remove(&mut self, key: &K) -> Result<(), DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity;

        loop {
            let record = self.record(pos)?;

            match record.meta_value {
                MetaValue::Empty => return Ok(()),
                MetaValue::Valid if record.key == *key => return self.remove_record(pos),
                MetaValue::Valid | MetaValue::Deleted => pos = self.next_pos(pos),
            }
        }
    }

    pub(crate) fn reserve(&mut self, new_capacity: u64) -> Result<(), DbError> {
        if self.capacity < new_capacity {
            return self.rehash(new_capacity);
        }

        Ok(())
    }

    pub(crate) fn size(&self) -> u64 {
        self.size
    }

    pub(crate) fn storage_index(&self) -> i64 {
        self.storage_index
    }

    pub(crate) fn to_hash_map(&self) -> Result<HashMap<K, T>, DbError> {
        let mut map = HashMap::<K, T>::new();
        map.reserve(self.size as usize);

        let data: StorageHashMapData<K, T> = self.storage.borrow_mut().value(self.storage_index)?;

        for record in data.data {
            if record.meta_value == MetaValue::Valid {
                map.insert(record.key, record.value);
            }
        }

        Ok(map)
    }

    pub(crate) fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity;

        loop {
            let record = self.record(pos)?;

            match record.meta_value {
                MetaValue::Empty => return Ok(None),
                MetaValue::Valid if record.key == *key => return Ok(Some(record.value)),
                MetaValue::Valid | MetaValue::Deleted => pos = self.next_pos(pos),
            }
        }
    }

    fn capacity_from_bytes(len: u64) -> u64 {
        (len - u64::serialized_size()) / T::serialized_size()
    }

    fn ensure_capacity(&mut self, new_capacity: u64) -> bool {
        let old_capacity = self.capacity;

        if new_capacity < 64 {
            self.capacity = 64;
        } else {
            self.capacity = new_capacity;
        }

        old_capacity != self.capacity
    }

    fn find_or_free(&mut self, key: &K) -> Result<(u64, StorageHashMapKeyValue<K, T>), DbError> {
        if self.max_size() < (self.size + 1) {
            self.rehash(self.capacity * 2)?;
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity;

        loop {
            let record = self.record(pos)?;

            match record.meta_value {
                MetaValue::Empty => return Ok((pos, record)),
                MetaValue::Valid if record.key == *key => return Ok((pos, record)),
                MetaValue::Valid | MetaValue::Deleted => pos = self.next_pos(pos),
            }
        }
    }

    fn insert_meta_value(&mut self, pos: u64, meta_value: MetaValue) -> Result<(), DbError> {
        let offset = Self::record_offset(pos) + StorageHashMapKeyValue::<K, T>::meta_value_offset();

        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, offset, &meta_value)
    }

    fn insert_value(&mut self, pos: u64, key: K, value: T) -> Result<(), DbError> {
        let record = StorageHashMapKeyValue {
            key,
            value,
            meta_value: MetaValue::Valid,
        };
        let offset = Self::record_offset(pos);

        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, offset, &record)
    }

    fn max_size(&self) -> u64 {
        self.capacity * 15 / 16
    }

    fn min_size(&self) -> u64 {
        self.capacity * 7 / 16
    }

    fn next_pos(&self, pos: u64) -> u64 {
        if pos == self.capacity - 1 {
            0
        } else {
            pos + 1
        }
    }

    fn place_new_record(
        &self,
        new_data: &mut StorageHashMapData<K, T>,
        record: StorageHashMapKeyValue<K, T>,
    ) {
        let hash = record.key.stable_hash();
        let mut pos = hash % self.capacity;

        while new_data.data[pos as usize].meta_value != MetaValue::Empty {
            pos = self.next_pos(pos);
        }

        new_data.data[pos as usize] = record;
    }

    fn record(&self, pos: u64) -> Result<StorageHashMapKeyValue<K, T>, DbError> {
        let offset = Self::record_offset(pos);

        self.storage
            .borrow_mut()
            .value_at::<StorageHashMapKeyValue<K, T>>(self.storage_index, offset)
    }

    fn record_offset(pos: u64) -> u64 {
        u64::serialized_size() + StorageHashMapKeyValue::<K, T>::serialized_size() * pos
    }

    fn rehash(&mut self, new_capacity: u64) -> Result<(), DbError> {
        if self.ensure_capacity(new_capacity) {
            let mut store = self.storage.borrow_mut();
            let old_data: StorageHashMapData<K, T> = store.value(self.storage_index)?;
            store.insert_at(self.storage_index, 0, &self.rehash_old_data(old_data))?;
            store.resize_value(self.storage_index, Self::record_offset(self.capacity))?;
        }

        Ok(())
    }

    fn rehash_old_data(&self, old_data: StorageHashMapData<K, T>) -> StorageHashMapData<K, T> {
        let mut new_data = StorageHashMapData::<K, T> {
            data: vec![StorageHashMapKeyValue::<K, T>::default(); self.capacity as usize],
            size: old_data.size,
        };

        for record in old_data.data {
            if record.meta_value == MetaValue::Valid {
                self.place_new_record(&mut new_data, record);
            }
        }

        new_data
    }

    fn remove_record(&mut self, pos: u64) -> Result<(), DbError> {
        self.storage.borrow_mut().transaction();
        self.insert_meta_value(pos, MetaValue::Deleted)?;
        self.set_size(self.size - 1)?;

        if 0 != self.size && (self.size - 1) < self.min_size() {
            self.rehash(self.capacity / 2)?;
        }

        self.storage.borrow_mut().commit()
    }

    fn set_size(&mut self, new_size: u64) -> Result<(), DbError> {
        self.size = new_size;
        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, 0, &self.size)
    }
}

impl<K, T, Data> TryFrom<std::rc::Rc<std::cell::RefCell<Storage<Data>>>>
    for StorageHashMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: StorageData,
{
    type Error = DbError;

    fn try_from(
        storage: std::rc::Rc<std::cell::RefCell<Storage<Data>>>,
    ) -> Result<Self, Self::Error> {
        let index = storage.borrow_mut().insert(&StorageHashMapData::<K, T> {
            data: vec![StorageHashMapKeyValue::<K, T>::default()],
            size: 0,
        })?;

        Ok(Self {
            storage,
            storage_index: index,
            size: 0,
            capacity: 1,
            phantom_data: std::marker::PhantomData::<(K, T)>,
        })
    }
}

impl<K, T, Data> TryFrom<(std::rc::Rc<std::cell::RefCell<Storage<Data>>>, i64)>
    for StorageHashMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: StorageData,
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (std::rc::Rc<std::cell::RefCell<Storage<Data>>>, i64),
    ) -> Result<Self, Self::Error> {
        let byte_size = storage_with_index
            .0
            .borrow_mut()
            .value_size(storage_with_index.1)?;
        let size = storage_with_index
            .0
            .borrow_mut()
            .value_at::<u64>(storage_with_index.1, 0)?;
        let capacity = Self::capacity_from_bytes(byte_size);

        Ok(Self {
            storage: storage_with_index.0,
            storage_index: storage_with_index.1,
            size,
            capacity,
            phantom_data: std::marker::PhantomData::<(K, T)>,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::file_storage::FileStorage;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();

        assert_eq!(map.size(), 3);
        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(15)));
        assert_eq!(map.value(&7), Ok(Some(20)));
    }

    #[test]
    fn insert_reallocate() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

        assert_eq!(map.capacity(), 1);

        for i in 0..100 {
            map.insert(i, i).unwrap();
        }

        assert_eq!(map.size(), 100);
        assert_eq!(map.capacity(), 128);

        for i in 0..100 {
            assert_eq!(map.value(&i), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_reallocate_with_collisions() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

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
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

        assert_eq!(map.insert(1, 10), Ok(None));
        assert_eq!(map.insert(5, 15), Ok(None));
        assert_eq!(map.size(), 2);
        assert_eq!(map.insert(5, 20), Ok(Some(15)));
        assert_eq!(map.size(), 2);

        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(20)));
    }

    #[test]
    fn remove() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();

        assert_eq!(map.size(), 3);
        map.remove(&5).unwrap();

        assert_eq!(map.size(), 2);
        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(None));
        assert_eq!(map.value(&7), Ok(Some(20)));
    }

    #[test]
    fn remove_deleted() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();

        assert_eq!(map.size(), 3);

        map.remove(&5).unwrap();

        assert_eq!(map.size(), 2);
        assert_eq!(map.value(&5), Ok(None));

        map.remove(&5).unwrap();

        assert_eq!(map.size(), 2);
    }

    #[test]
    fn remove_missing() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

        assert_eq!(map.size(), 0);
        assert_eq!(map.remove(&0), Ok(()));
        assert_eq!(map.size(), 0);
    }

    #[test]
    fn remove_shrinks_capacity() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

        for i in 0..100 {
            map.insert(i, i).unwrap();
        }

        assert_eq!(map.size(), 100);
        assert_eq!(map.capacity(), 128);

        for i in 0..100 {
            map.remove(&i).unwrap();
        }

        assert_eq!(map.size(), 0);
        assert_eq!(map.capacity(), 64);
    }

    #[test]
    fn reserve_larger() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();
        map.insert(1, 1).unwrap();

        let capacity = map.capacity() + 10;
        let size = map.size();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.size(), size);
        assert_eq!(map.value(&1), Ok(Some(1)));
    }

    #[test]
    fn reserve_same() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();
        map.insert(1, 1).unwrap();

        let capacity = map.capacity();
        let size = map.size();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.size(), size);
    }

    #[test]
    fn reserve_smaller() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();
        map.insert(1, 1).unwrap();

        let current_capacity = map.capacity();
        let capacity = current_capacity - 10;
        let size = map.size();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), current_capacity);
        assert_eq!(map.size(), size);
    }

    #[test]
    fn try_from_storage_index() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let index;

        {
            let mut map = StorageHashMap::<i64, i64>::try_from(storage.clone()).unwrap();
            map.insert(1, 1).unwrap();
            map.insert(3, 2).unwrap();
            map.insert(5, 3).unwrap();
            map.remove(&3).unwrap();
            index = map.storage_index();
        }

        let map = StorageHashMap::<i64, i64>::try_from((storage, index)).unwrap();

        let mut expected = HashMap::<i64, i64>::new();
        expected.insert(1, 1);
        expected.insert(5, 3);

        assert_eq!(map.to_hash_map(), Ok(expected));
    }

    #[test]
    fn try_from_storage_missing_index() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        assert_eq!(
            StorageHashMap::<i64, i64>::try_from((storage, 1))
                .err()
                .unwrap(),
            DbError::from("index '1' not found")
        );
    }

    #[test]
    fn to_hash_map() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();
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
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();
        let other = map.to_hash_map().unwrap();

        assert_eq!(other.len(), 0);
    }

    #[test]
    fn value_missing() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

        assert_eq!(map.value(&0), Ok(None));
    }

    #[test]
    fn values_at_end() {
        let test_file = TestFile::new();
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(127, 10).unwrap();
        map.insert(255, 11).unwrap();
        map.insert(191, 12).unwrap();

        assert_eq!(map.value(&127), Ok(Some(10)));
        assert_eq!(map.value(&255), Ok(Some(11)));
        assert_eq!(map.value(&191), Ok(Some(12)));
    }
}
