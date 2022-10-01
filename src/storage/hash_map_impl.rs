use super::hash_map_data::HashMapData;
use super::hash_map_iterator::HashMapIterator;
use super::hash_map_key_value::HashMapKeyValue;
use super::hash_map_meta_value::HashMapMetaValue;
use super::Serialize;
use super::StableHash;
use crate::DbError;
use std::hash::Hash;

pub(crate) struct HashMapImpl<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: HashMapData<K, T>,
{
    pub(super) data: Data,
    pub(super) phantom_data: std::marker::PhantomData<(K, T)>,
}

#[allow(dead_code)]
impl<K, T, Data> HashMapImpl<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: HashMapData<K, T>,
{
    pub(crate) fn capacity(&self) -> u64 {
        self.data.capacity()
    }

    pub(crate) fn count(&self) -> u64 {
        self.data.count()
    }

    pub(crate) fn insert(&mut self, key: K, value: T) -> Result<Option<T>, DbError> {
        self.data.transaction();
        let free = self.find_or_free(&key)?;
        self.insert_value(free.0, key, value)?;

        if free.1.is_none() {
            self.data.set_count(self.data.count() + 1)?;
        }

        self.data.commit()?;

        Ok(free.1)
    }

    pub(crate) fn iter(&self) -> HashMapIterator<K, T, Data> {
        HashMapIterator::<K, T, Data> {
            pos: 0,
            data: &self.data,
            phantom_data: std::marker::PhantomData,
        }
    }

    pub(crate) fn remove(&mut self, key: &K) -> Result<(), DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        self.data.transaction();

        loop {
            let record = self.data.record(pos)?;

            match record.meta_value {
                HashMapMetaValue::Empty => break,
                HashMapMetaValue::Valid if record.key == *key => {
                    self.remove_record(pos)?;
                }
                HashMapMetaValue::Valid | HashMapMetaValue::Deleted => {
                    pos = Self::next_pos(pos, self.capacity());
                }
            }
        }

        self.data.commit()
    }

    pub(crate) fn reserve(&mut self, new_capacity: u64) -> Result<(), DbError> {
        if self.capacity() < new_capacity {
            return self.rehash(new_capacity);
        }

        Ok(())
    }

    pub(crate) fn to_hash_map(&self) -> Result<std::collections::HashMap<K, T>, DbError> {
        let mut map = std::collections::HashMap::<K, T>::new();
        map.reserve(self.count() as usize);

        for i in 0..self.capacity() {
            let record = self.data.record(i)?;

            if record.meta_value == HashMapMetaValue::Valid {
                map.insert(record.key, record.value);
            }
        }

        Ok(map)
    }

    pub(crate) fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            let record = self.data.record(pos)?;

            match record.meta_value {
                HashMapMetaValue::Empty => return Ok(None),
                HashMapMetaValue::Valid if record.key == *key => return Ok(Some(record.value)),
                HashMapMetaValue::Valid | HashMapMetaValue::Deleted => {
                    pos = Self::next_pos(pos, self.capacity())
                }
            }
        }
    }

    fn find_or_free(&mut self, key: &K) -> Result<(u64, Option<T>), DbError> {
        if self.max_size() < (self.data.count() + 1) {
            self.rehash(self.capacity() * 2)?;
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            let record = self.data.record(pos)?;

            match record.meta_value {
                HashMapMetaValue::Empty | HashMapMetaValue::Deleted => return Ok((pos, None)),
                HashMapMetaValue::Valid if record.key == *key => {
                    return Ok((pos, Some(record.value)))
                }
                HashMapMetaValue::Valid => pos = Self::next_pos(pos, self.capacity()),
            }
        }
    }

    pub(super) fn insert_value(&mut self, pos: u64, key: K, value: T) -> Result<(), DbError> {
        self.data.set_value(
            pos,
            HashMapKeyValue {
                meta_value: HashMapMetaValue::Valid,
                key,
                value,
            },
        )
    }

    pub(super) fn max_size(&self) -> u64 {
        self.capacity() * 15 / 16
    }

    pub(super) fn min_size(&self) -> u64 {
        self.capacity() * 7 / 16
    }

    pub(super) fn next_pos(pos: u64, capacity: u64) -> u64 {
        if pos == capacity - 1 {
            0
        } else {
            pos + 1
        }
    }

    fn place_new_record(
        &self,
        new_data: &mut Vec<HashMapKeyValue<K, T>>,
        record: HashMapKeyValue<K, T>,
    ) {
        let hash = record.key.stable_hash();
        let mut pos = hash % new_data.len() as u64;

        while new_data[pos as usize].meta_value != HashMapMetaValue::Empty {
            pos = Self::next_pos(pos, new_data.len() as u64);
        }

        new_data[pos as usize] = record;
    }

    pub(super) fn rehash(&mut self, mut new_capacity: u64) -> Result<(), DbError> {
        new_capacity = std::cmp::max(new_capacity, 64);

        if new_capacity != self.capacity() {
            let old_data = self.data.values()?;
            self.data.transaction();
            self.data
                .set_values(self.rehash_old_data(old_data, new_capacity))?;
            self.data.commit()?;
        }

        Ok(())
    }

    fn rehash_old_data(
        &self,
        old_data: Vec<HashMapKeyValue<K, T>>,
        new_capacity: u64,
    ) -> Vec<HashMapKeyValue<K, T>> {
        let mut new_data: Vec<HashMapKeyValue<K, T>> =
            vec![HashMapKeyValue::<K, T>::default(); new_capacity as usize];

        for record in old_data {
            if record.meta_value == HashMapMetaValue::Valid {
                self.place_new_record(&mut new_data, record);
            }
        }

        new_data
    }

    pub(super) fn remove_record(&mut self, pos: u64) -> Result<(), DbError> {
        self.data.set_meta_value(pos, HashMapMetaValue::Deleted)?;
        self.data.set_count(self.data.count() - 1)?;

        if 0 != self.data.count() && (self.data.count() - 1) < self.min_size() {
            self.rehash(self.capacity() / 2)?;
        }

        Ok(())
    }
}
