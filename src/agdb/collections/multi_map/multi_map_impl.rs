use super::MapCommon;
use crate::collections::map_common::map_data::MapData;
use crate::collections::map_common::map_iterator::MapIterator;
use crate::collections::map_common::map_value_state::MapValueState;
use crate::db::db_error::DbError;
use crate::utilities::serialize::OldSerialize;
use crate::utilities::stable_hash::StableHash;
use std::hash::Hash;

pub struct MultiMapImpl<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + OldSerialize,
    T: Clone + Default + Eq + PartialEq + OldSerialize,
    Data: MapData<K, T>,
{
    pub(crate) map_common: MapCommon<K, T, Data>,
}

#[allow(dead_code)]
impl<K, T, Data> MultiMapImpl<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + OldSerialize,
    T: Clone + Default + Eq + PartialEq + OldSerialize,
    Data: MapData<K, T>,
{
    pub fn capacity(&self) -> u64 {
        self.map_common.capacity()
    }

    pub fn count(&self) -> u64 {
        self.map_common.count()
    }

    pub fn insert(&mut self, key: K, value: T) -> Result<(), DbError> {
        self.map_common.data.transaction();
        let free = self.find_free(&key)?;
        self.map_common.insert_value(free, key, value)?;
        self.map_common.data.set_count(self.count() + 1)?;
        self.map_common.data.commit()
    }

    pub fn iter(&self) -> MapIterator<K, T, Data> {
        self.map_common.iter()
    }

    pub fn remove_key(&mut self, key: &K) -> Result<(), DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.map_common.data.capacity();

        self.map_common.data.transaction();

        loop {
            let record = self.map_common.data.record(pos)?;

            match record.state {
                MapValueState::Empty => break,
                MapValueState::Valid if record.key == *key => {
                    self.map_common.remove_record(pos)?;
                }
                MapValueState::Valid | MapValueState::Deleted => {
                    pos = MapCommon::<K, T, Data>::next_pos(pos, self.capacity());
                }
            }

            pos = MapCommon::<K, T, Data>::next_pos(pos, self.capacity());
        }

        self.map_common.data.commit()
    }

    pub fn remove_value(&mut self, key: &K, value: &T) -> Result<(), DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            let record = self.map_common.data.record(pos)?;

            match record.state {
                MapValueState::Empty => break,
                MapValueState::Valid if record.key == *key && record.value == *value => {
                    self.map_common.data.transaction();
                    self.map_common.remove_record(pos)?;
                    self.map_common.data.commit()?;
                    break;
                }
                MapValueState::Valid | MapValueState::Deleted => {
                    pos = MapCommon::<K, T, Data>::next_pos(pos, self.capacity());
                }
            }
        }

        Ok(())
    }

    pub fn reserve(&mut self, new_capacity: u64) -> Result<(), DbError> {
        self.map_common.reserve(new_capacity)
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        self.map_common.value(key)
    }

    pub fn values(&self, key: &K) -> Result<Vec<T>, DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let mut values: Vec<T> = vec![];

        loop {
            let record = self.map_common.data.record(pos)?;

            match record.state {
                MapValueState::Empty => break,
                MapValueState::Valid if record.key == *key => values.push(record.value.clone()),
                MapValueState::Valid | MapValueState::Deleted => {}
            }

            pos = MapCommon::<K, T, Data>::next_pos(pos, self.capacity());
        }

        Ok(values)
    }

    fn find_free(&mut self, key: &K) -> Result<u64, DbError> {
        if self.map_common.max_size() < (self.count() + 1) {
            self.map_common.rehash(self.capacity() * 2)?;
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            let meta_value = self.map_common.data.meta_value(pos)?;

            match meta_value {
                MapValueState::Empty | MapValueState::Deleted => return Ok(pos),
                MapValueState::Valid => {
                    pos = MapCommon::<K, T, Data>::next_pos(pos, self.capacity());
                }
            }
        }
    }
}
