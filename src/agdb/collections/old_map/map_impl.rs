use crate::collections::old_map_common::map_data::MapData;
use crate::collections::old_map_common::map_iterator::MapIterator;
use crate::collections::old_map_common::map_value_state::MapValueState;
use crate::collections::old_map_common::MapCommon;
use crate::db::db_error::DbError;
use crate::utilities::old_serialize::OldSerialize;
use crate::utilities::stable_hash::StableHash;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct OldMapImpl<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + OldSerialize,
    T: Clone + Default + OldSerialize,
    Data: MapData<K, T>,
{
    pub(crate) map_common: MapCommon<K, T, Data>,
    pub(crate) phantom_data: PhantomData<(K, T)>,
}

#[allow(dead_code)]
impl<K, T, Data> OldMapImpl<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + OldSerialize,
    T: Clone + Default + OldSerialize,
    Data: MapData<K, T>,
{
    pub fn capacity(&self) -> u64 {
        self.map_common.capacity()
    }

    pub fn count(&self) -> u64 {
        self.map_common.count()
    }

    pub fn insert(&mut self, key: K, value: T) -> Result<Option<T>, DbError> {
        self.map_common.data.transaction();
        let free = self.find_or_free(&key)?;
        self.map_common.insert_value(free.0, key, value)?;

        if free.1.is_none() {
            self.map_common
                .data
                .set_count(self.map_common.count() + 1)?;
        }

        self.map_common.data.commit()?;

        Ok(free.1)
    }

    pub fn iter(&self) -> MapIterator<K, T, Data> {
        self.map_common.iter()
    }

    pub fn remove(&mut self, key: &K) -> Result<(), DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

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
        }

        self.map_common.data.commit()
    }

    pub fn reserve(&mut self, new_capacity: u64) -> Result<(), DbError> {
        self.map_common.reserve(new_capacity)
    }

    pub fn to_hash_map(&self) -> Result<HashMap<K, T>, DbError> {
        let mut map = HashMap::<K, T>::new();
        map.reserve(self.count() as usize);

        for i in 0..self.capacity() {
            let record = self.map_common.data.record(i)?;

            if record.state == MapValueState::Valid {
                map.insert(record.key, record.value);
            }
        }

        Ok(map)
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        self.map_common.value(key)
    }

    fn find_or_free(&mut self, key: &K) -> Result<(u64, Option<T>), DbError> {
        if self.map_common.max_size() < (self.count() + 1) {
            self.map_common.rehash(self.capacity() * 2)?;
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            let record = self.map_common.data.record(pos)?;

            match record.state {
                MapValueState::Empty | MapValueState::Deleted => return Ok((pos, None)),
                MapValueState::Valid if record.key == *key => return Ok((pos, Some(record.value))),
                MapValueState::Valid => {
                    pos = MapCommon::<K, T, Data>::next_pos(pos, self.capacity())
                }
            }
        }
    }
}
