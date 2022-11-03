pub mod map_data;
pub mod map_data_memory;
pub mod map_data_storage;
pub mod map_iterator;
pub mod map_value;
pub mod map_value_state;

use self::map_data::MapData;
use self::map_iterator::MapIterator;
use self::map_value::MapValue;
use self::map_value_state::MapValueState;
use crate::db::db_error::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::stable_hash::StableHash;
use std::cmp::max;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct MapCommon<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: MapData<K, T>,
{
    pub data: Data,
    pub(crate) phantom_data: PhantomData<(K, T)>,
}

#[allow(dead_code)]
impl<K, T, Data> MapCommon<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: MapData<K, T>,
{
    pub fn capacity(&self) -> u64 {
        self.data.capacity()
    }

    pub fn count(&self) -> u64 {
        self.data.count()
    }

    pub fn iter(&self) -> MapIterator<K, T, Data> {
        MapIterator::<K, T, Data> {
            pos: 0,
            data: &self.data,
            phantom_data: PhantomData,
        }
    }

    pub fn reserve(&mut self, new_capacity: u64) -> Result<(), DbError> {
        if self.capacity() < new_capacity {
            return self.rehash(new_capacity);
        }

        Ok(())
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            let record = self.data.record(pos)?;

            match record.state {
                MapValueState::Empty => return Ok(None),
                MapValueState::Valid if record.key == *key => return Ok(Some(record.value)),
                MapValueState::Valid | MapValueState::Deleted => {
                    pos = Self::next_pos(pos, self.capacity())
                }
            }
        }
    }

    pub fn insert_value(&mut self, pos: u64, key: K, value: T) -> Result<(), DbError> {
        self.data.set_value(
            pos,
            MapValue {
                state: MapValueState::Valid,
                key,
                value,
            },
        )
    }

    pub fn max_size(&self) -> u64 {
        self.capacity() * 15 / 16
    }

    pub fn next_pos(pos: u64, capacity: u64) -> u64 {
        if pos == capacity - 1 {
            0
        } else {
            pos + 1
        }
    }

    pub fn rehash(&mut self, mut new_capacity: u64) -> Result<(), DbError> {
        new_capacity = max(new_capacity, 64);

        if new_capacity != self.capacity() {
            let old_data = self.data.take_values()?;
            self.data.transaction();
            self.data
                .set_values(self.rehash_old_data(old_data, new_capacity))?;
            self.data.commit()?;
        }

        Ok(())
    }

    pub fn remove_record(&mut self, pos: u64) -> Result<(), DbError> {
        self.data.set_meta_value(pos, MapValueState::Deleted)?;
        self.data.set_count(self.data.count() - 1)?;

        if 0 != self.data.count() && (self.data.count() - 1) < self.min_size() {
            self.rehash(self.capacity() / 2)?;
        }

        Ok(())
    }

    fn min_size(&self) -> u64 {
        self.capacity() * 7 / 16
    }

    fn place_new_record(&self, new_data: &mut Vec<MapValue<K, T>>, record: MapValue<K, T>) {
        let hash = record.key.stable_hash();
        let mut pos = hash % new_data.len() as u64;

        while new_data[pos as usize].state != MapValueState::Empty {
            pos = Self::next_pos(pos, new_data.len() as u64);
        }

        new_data[pos as usize] = record;
    }

    fn rehash_old_data(
        &self,
        old_data: Vec<MapValue<K, T>>,
        new_capacity: u64,
    ) -> Vec<MapValue<K, T>> {
        let mut new_data: Vec<MapValue<K, T>> =
            vec![MapValue::<K, T>::default(); new_capacity as usize];

        for record in old_data {
            if record.state == MapValueState::Valid {
                self.place_new_record(&mut new_data, record);
            }
        }

        new_data
    }
}

impl<K, T, Data> From<Data> for MapCommon<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: MapData<K, T>,
{
    fn from(data: Data) -> Self {
        Self {
            data,
            phantom_data: PhantomData,
        }
    }
}
