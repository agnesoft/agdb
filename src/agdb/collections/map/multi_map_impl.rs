use crate::collections::map::map_data::MapData;
use crate::collections::map::map_iterator::MapIterator;
use crate::collections::map::map_value_state::MapValueState;
use crate::utilities::stable_hash::StableHash;
use crate::DbError;
use std::cmp::max;
use std::hash::Hash;
use std::marker::PhantomData;

pub struct MultiMapImpl<K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + PartialEq,
    Data: MapData<K, T>,
{
    pub(crate) data: Data,
    pub(crate) phantom_marker: PhantomData<(K, T)>,
}

impl<K, T, Data> MultiMapImpl<K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + PartialEq,
    Data: MapData<K, T>,
{
    pub fn capacity(&self) -> u64 {
        self.data.capacity()
    }

    pub fn contains(&self, key: &K, value: &T) -> Result<bool, DbError> {
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
        self.data.transaction();
        let index = self.free_index(key)?;
        self.data.set_state(index, MapValueState::Valid)?;
        self.data.set_key(index, key)?;
        self.data.set_value(index, value)?;
        self.data.set_len(self.len() + 1)?;
        self.data.commit()
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

        self.data.transaction();

        loop {
            match self.data.state(pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid if self.data.key(pos)? == *key => {
                    self.data.set_state(pos, MapValueState::Deleted)?;
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

        self.data.commit()
    }

    pub fn remove_value(&mut self, key: &K, value: &T) -> Result<(), DbError> {
        if self.capacity() == 0 {
            return Ok(());
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        self.data.transaction();

        loop {
            match self.data.state(pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid
                    if self.data.key(pos)? == *key && self.data.value(pos)? == *value =>
                {
                    self.data.set_state(pos, MapValueState::Deleted)?;
                    self.data.set_len(self.len() - 1)?;

                    if self.len() <= self.min_len() {
                        self.rehash(self.capacity() / 2)?;
                    }

                    break;
                }
                MapValueState::Valid | MapValueState::Deleted => pos = self.next_pos(pos),
            }
        }

        self.data.commit()
    }

    pub fn replace(&mut self, key: &K, value: &T, new_value: &T) -> Result<(), DbError> {
        if self.capacity() == 0 {
            return Ok(());
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        self.data.transaction();

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

        self.data.commit()
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
            self.data.set_state(*i, MapValueState::Empty)?;
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

    fn rehash_values(&mut self, current_capacity: u64, new_capacity: u64) -> Result<(), DbError> {
        let mut i = 0_u64;
        let mut empty_list = vec![true; new_capacity as usize];

        while i != current_capacity {
            self.rehash_value(self.data.state(i)?, &mut i, new_capacity, &mut empty_list)?;
        }

        Ok(())
    }

    fn shrink(&mut self, current_capacity: u64, new_capacity: u64) -> Result<(), DbError> {
        self.rehash_values(current_capacity, new_capacity)?;
        self.data.resize(new_capacity)
    }
}
