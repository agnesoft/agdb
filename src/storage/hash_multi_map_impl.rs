use super::hash_map_data::HashMapData;
use super::hash_map_impl::HashMapImpl;
use super::hash_map_iterator::HashMapIterator;
use super::hash_map_meta_value::HashMapMetaValue;
use super::StableHash;
use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use std::hash::Hash;

pub(crate) struct HashMultiMapImpl<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
    Data: HashMapData<K, T>,
{
    pub(super) map: HashMapImpl<K, T, Data>,
}

#[allow(dead_code)]
impl<K, T, Data> HashMultiMapImpl<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
    Data: HashMapData<K, T>,
{
    pub(crate) fn capacity(&self) -> u64 {
        self.map.capacity()
    }

    pub(crate) fn count(&self) -> u64 {
        self.map.count()
    }

    pub(crate) fn insert(&mut self, key: K, value: T) -> Result<(), DbError> {
        self.map.data.transaction();
        let free = self.find_free(&key)?;
        self.map.insert_value(free, key, value)?;
        self.map.data.set_count(self.count() + 1)?;
        self.map.data.commit()
    }

    pub(crate) fn iter(&self) -> HashMapIterator<K, T, Data> {
        self.map.iter()
    }

    pub(crate) fn remove_key(&mut self, key: &K) -> Result<(), DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.map.data.capacity();

        self.map.data.transaction();

        loop {
            let record = self.map.data.record(pos)?;

            match record.meta_value {
                HashMapMetaValue::Empty => break,
                HashMapMetaValue::Valid if record.key == *key => {
                    self.map.remove_record(pos)?;
                }
                HashMapMetaValue::Valid | HashMapMetaValue::Deleted => {
                    pos = HashMapImpl::<K, T, Data>::next_pos(pos, self.capacity());
                }
            }

            pos = HashMapImpl::<K, T, Data>::next_pos(pos, self.capacity());
        }

        self.map.data.commit()
    }

    pub(crate) fn remove_value(&mut self, key: &K, value: &T) -> Result<(), DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            let record = self.map.data.record(pos)?;

            match record.meta_value {
                HashMapMetaValue::Empty => break,
                HashMapMetaValue::Valid if record.key == *key && record.value == *value => {
                    self.map.data.transaction();
                    self.map.remove_record(pos)?;
                    self.map.data.commit()?;
                    break;
                }
                HashMapMetaValue::Valid | HashMapMetaValue::Deleted => {
                    pos = HashMapImpl::<K, T, Data>::next_pos(pos, self.capacity());
                }
            }
        }

        Ok(())
    }

    pub(crate) fn reserve(&mut self, new_capacity: u64) -> Result<(), DbError> {
        self.map.reserve(new_capacity)
    }

    pub(crate) fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        self.map.value(key)
    }

    pub(crate) fn values(&self, key: &K) -> Result<Vec<T>, DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let mut values: Vec<T> = vec![];

        loop {
            let record = self.map.data.record(pos)?;

            match record.meta_value {
                HashMapMetaValue::Empty => break,
                HashMapMetaValue::Valid if record.key == *key => values.push(record.value.clone()),
                HashMapMetaValue::Valid | HashMapMetaValue::Deleted => {}
            }

            pos = HashMapImpl::<K, T, Data>::next_pos(pos, self.capacity());
        }

        Ok(values)
    }

    fn find_free(&mut self, key: &K) -> Result<u64, DbError> {
        if self.map.max_size() < (self.count() + 1) {
            self.map.rehash(self.capacity() * 2)?;
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            let meta_value = self.map.data.meta_value(pos)?;

            match meta_value {
                HashMapMetaValue::Empty | HashMapMetaValue::Deleted => return Ok(pos),
                HashMapMetaValue::Valid => {
                    pos = HashMapImpl::<K, T, Data>::next_pos(pos, self.capacity());
                }
            }
        }
    }
}
