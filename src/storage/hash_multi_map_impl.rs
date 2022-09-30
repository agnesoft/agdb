use super::hash_map_data::HashMapData;
use super::hash_map_impl::HashMapImpl;
use super::hash_map_iterator::HashMapIterator;
use super::hash_map_meta_value::HashMapMetaValue;
use super::Serialize;
use super::StableHash;
use crate::DbError;
use std::hash::Hash;

pub(crate) struct HashMultiMapImpl<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
    Data: HashMapData<K, T>,
{
    multi_map: HashMapImpl<K, T, Data>,
}

#[allow(dead_code)]
impl<K, T, Data> HashMultiMapImpl<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
    Data: HashMapData<K, T>,
{
    pub(crate) fn capacity(&self) -> u64 {
        self.multi_map.capacity()
    }

    pub(crate) fn count(&self) -> u64 {
        self.multi_map.count()
    }

    pub(crate) fn insert(&mut self, key: K, value: T) -> Result<(), DbError> {
        self.multi_map.data.transaction();
        let free = self.find_free(&key)?;
        self.multi_map.insert_value(free, key, value)?;
        self.multi_map.data.set_count(self.count() + 1)?;
        self.multi_map.data.commit()
    }

    pub(crate) fn iter(&self) -> HashMapIterator {
        todo!()
    }

    pub(crate) fn remove_key(&mut self, key: &K) -> Result<(), DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.multi_map.data.capacity();

        self.multi_map.data.transaction();

        loop {
            let record = self.multi_map.data.record(pos)?;

            match record.meta_value {
                HashMapMetaValue::Empty => break,
                HashMapMetaValue::Valid if record.key == *key => {
                    self.multi_map.remove_record(pos)?;
                }
                HashMapMetaValue::Valid | HashMapMetaValue::Deleted => {
                    pos = self.multi_map.next_pos(pos);
                }
            }
        }

        self.multi_map.data.commit()
    }

    pub(crate) fn remove_value(&mut self, key: &K, value: &T) -> Result<(), DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            let record = self.multi_map.data.record(pos)?;

            match record.meta_value {
                HashMapMetaValue::Empty => break,
                HashMapMetaValue::Valid if record.key == *key && record.value == *value => {
                    self.multi_map.data.transaction();
                    self.multi_map.remove_record(pos)?;
                    self.multi_map.data.commit()?;
                }
                HashMapMetaValue::Valid | HashMapMetaValue::Deleted => {
                    pos = self.multi_map.next_pos(pos);
                }
            }
        }

        Ok(())
    }

    pub(crate) fn reserve(&mut self, new_capacity: u64) -> Result<(), DbError> {
        self.multi_map.reserve(new_capacity)
    }

    pub(crate) fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        self.multi_map.value(key)
    }

    pub(crate) fn values(&self, key: &K) -> Result<Option<Vec<T>>, DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let mut values: Vec<T> = vec![];

        loop {
            let record = self.multi_map.data.record(pos)?;

            match record.meta_value {
                HashMapMetaValue::Empty => break,
                HashMapMetaValue::Valid if record.key == *key => values.push(record.value.clone()),
                HashMapMetaValue::Valid | HashMapMetaValue::Deleted => {
                    pos = self.multi_map.next_pos(pos)
                }
            }
        }

        Ok(Some(values))
    }

    fn find_free(&mut self, key: &K) -> Result<u64, DbError> {
        if self.multi_map.max_size() < (self.count() + 1) {
            self.multi_map.rehash(self.capacity() * 2)?;
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            let meta_value = self.multi_map.data.meta_value(pos)?;

            match meta_value {
                HashMapMetaValue::Empty | HashMapMetaValue::Deleted => return Ok(pos),
                HashMapMetaValue::Valid => pos = self.multi_map.next_pos(pos),
            }
        }
    }
}
