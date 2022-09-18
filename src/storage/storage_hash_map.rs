use super::file_storage::FileStorage;
use super::serialize::Serialize;
use super::stable_hash::StableHash;
use super::storage_hash_map_data::StorageHashMapData;
use super::storage_hash_map_key_value::StorageHashMapKeyValue;
use super::storage_hash_map_meta_value::MetaValue;
use super::Storage;
use crate::DbError;

#[allow(dead_code)]
pub(crate) struct StorageHashMap<K, T, S = FileStorage>
where
    K: Clone + Default + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    S: Storage,
{
    storage: std::rc::Rc<std::cell::RefCell<S>>,
    storage_index: i64,
    size: u64,
    capacity: u64,
    phantom_data: std::marker::PhantomData<(K, T)>,
}

#[allow(dead_code)]
impl<K, T, S> StorageHashMap<K, T, S>
where
    K: Clone + Default + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    S: Storage,
{
    pub(crate) fn insert(&mut self, key: K, value: T) -> Result<(), DbError> {
        self.storage.borrow_mut().transaction();
        let offset = self.free_offset(key.stable_hash())?;
        self.storage.borrow_mut().insert_at(
            self.storage_index,
            offset,
            &StorageHashMapKeyValue {
                key,
                value,
                meta_value: MetaValue::Valid,
            },
        )?;
        self.size += 1;
        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, 0, &self.size)?;
        self.storage.borrow_mut().commit()?;

        Ok(())
    }

    pub(crate) fn value(&mut self, key: &K) -> Result<Option<T>, DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity;

        loop {
            let offset = Self::record_offset(pos);
            let record = self
                .storage
                .borrow_mut()
                .value_at::<StorageHashMapKeyValue<K, T>>(self.storage_index, offset)?;

            match record.meta_value {
                MetaValue::Empty => return Ok(None),
                MetaValue::Valid if record.key == *key => return Ok(Some(record.value)),
                MetaValue::Valid | MetaValue::Deleted => pos = self.next_pos(pos),
            }
        }
    }

    fn free_offset(&mut self, hash: u64) -> Result<u64, DbError> {
        if (self.size + 1) > self.max_size() {
            self.rehash(self.capacity * 2)?;
        }

        let mut pos = hash % self.capacity;

        loop {
            let record_offset = Self::record_offset(pos);
            let offset = record_offset + StorageHashMapKeyValue::<K, T>::meta_value_offset();
            let meta_value = self
                .storage
                .borrow_mut()
                .value_at::<MetaValue>(self.storage_index, offset)?;

            match meta_value {
                MetaValue::Empty | MetaValue::Deleted => return Ok(record_offset),
                MetaValue::Valid => pos = self.next_pos(pos),
            }
        }
    }

    fn max_size(&self) -> u64 {
        self.capacity * 15 / 16
    }

    // fn min_size(&self) -> u64 {
    //     self.capacity * 7 / 16
    // }

    fn rehash(&mut self, new_capacity: u64) -> Result<(), DbError> {
        if new_capacity < 64 {
            self.capacity = 64;
        } else {
            self.capacity = new_capacity;
        }

        let old_data: StorageHashMapData<K, T> =
            self.storage.borrow_mut().value(self.storage_index)?;
        let mut new_data = StorageHashMapData::<K, T> {
            data: vec![StorageHashMapKeyValue::<K, T>::default(); self.capacity as usize],
            size: old_data.size,
        };

        for record in old_data.data {
            if record.meta_value == MetaValue::Valid {
                let hash = record.key.stable_hash();
                let mut pos = hash % self.capacity;

                loop {
                    if new_data.data[pos as usize].meta_value == MetaValue::Empty {
                        new_data.data[pos as usize] = record;
                        break;
                    }

                    pos = self.next_pos(pos);
                }
            }
        }

        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, 0, &new_data)?;
        self.storage
            .borrow_mut()
            .resize_value(self.storage_index, Self::record_offset(self.capacity))?;

        Ok(())
    }

    fn next_pos(&self, pos: u64) -> u64 {
        if pos == self.capacity - 1 {
            0
        } else {
            pos + 1
        }
    }

    fn record_offset(pos: u64) -> u64 {
        u64::serialized_size() as u64 + StorageHashMapKeyValue::<K, T>::serialized_size() * pos
    }
}

impl<K, T, S> TryFrom<std::rc::Rc<std::cell::RefCell<S>>> for StorageHashMap<K, T, S>
where
    K: Clone + Default + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    S: Storage,
{
    type Error = DbError;

    fn try_from(storage: std::rc::Rc<std::cell::RefCell<S>>) -> Result<Self, Self::Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn insert() {
        let test_file = TestFile::from("./storage_hash_map-insert.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

        map.insert(1, 10).unwrap();
        map.insert(5, 15).unwrap();
        map.insert(7, 20).unwrap();

        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(15)));
        assert_eq!(map.value(&7), Ok(Some(20)));
    }

    #[test]
    fn insert_reallocate() {
        let test_file = TestFile::from("./storage_hash_map-insert_reallocate.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

        for i in 0..100 {
            map.insert(i, i).unwrap();
        }

        for i in 0..100 {
            assert_eq!(map.value(&i), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_reallocate_with_collisions() {
        let test_file = TestFile::from("./storage_hash_map-insert_reallocate.agdb");
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
    fn value_missing() {
        let test_file = TestFile::from("./storage_hash_map-value_missing.agdb");
        let storage = std::rc::Rc::new(std::cell::RefCell::new(
            FileStorage::try_from(test_file.file_name().clone()).unwrap(),
        ));

        let mut map = StorageHashMap::<i64, i64>::try_from(storage).unwrap();

        assert_eq!(map.value(&0), Ok(None));
    }

    #[test]
    fn values_at_end() {
        let test_file = TestFile::from("./storage_hash_map-values_at_end.agdb");
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
