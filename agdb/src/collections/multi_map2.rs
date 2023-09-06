use super::map2::MapDataIndex;
use super::map2::MapIterator;
use super::map2::MapValueState;
use super::vec2::DbVec;
use super::vec2::VecValue;
use crate::storage::file_storage::FileStorage;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::stable_hash::StableHash;
use crate::DbError;

pub struct DbMultiMap<K, T, S = FileStorage>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    storage_index: StorageIndex,
    data_index: MapDataIndex,
    states: DbVec<MapValueState, S>,
    keys: DbVec<K, S>,
    values: DbVec<T, S>,
}

pub struct DbMultiMapImpl<'a, K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    multi_map: &'a DbMultiMap<K, T, S>,
    storage: &'a S,
}

pub struct DbMultiMapImplMut<'a, K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    multi_map: &'a mut DbMultiMap<K, T, S>,
    storage: &'a mut S,
}

pub struct MultiMapIterator<'a, K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    pub pos: u64,
    pub key: &'a K,
    pub multi_map: &'a DbMultiMap<K, T, S>,
    pub storage: &'a S,
}

impl<K, T, S> DbMultiMap<K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.states.len()
    }

    pub fn from_storage(storage: &S, storage_index: StorageIndex) -> Result<Self, DbError> {
        let data_index = storage.value::<MapDataIndex>(storage_index)?;
        let states = DbVec::<MapValueState, S>::from_storage(storage, data_index.states_index)?;
        let keys = DbVec::<K, S>::from_storage(storage, data_index.keys_index)?;
        let values = DbVec::<T, S>::from_storage(storage, data_index.values_index)?;

        Ok(Self {
            storage_index,
            data_index,
            states,
            keys,
            values,
        })
    }

    pub fn len(&self) -> u64 {
        self.data_index.len
    }

    pub fn new(storage: &mut S) -> Result<Self, DbError> {
        let states = DbVec::<MapValueState, S>::new(storage)?;
        let keys = DbVec::<K, S>::new(storage)?;
        let values = DbVec::<T, S>::new(storage)?;

        let data_index = MapDataIndex {
            len: 0,
            states_index: states.storage_index(),
            keys_index: keys.storage_index(),
            values_index: values.storage_index(),
        };

        let storage_index = storage.insert(&data_index)?;

        Ok(Self {
            storage_index,
            data_index,
            states,
            keys,
            values,
        })
    }

    pub fn read<'a>(&'a self, storage: &'a S) -> DbMultiMapImpl<'a, K, T, S> {
        DbMultiMapImpl {
            multi_map: self,
            storage,
        }
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_index
    }

    pub fn write<'a>(&'a mut self, storage: &'a mut S) -> DbMultiMapImplMut<'a, K, T, S> {
        DbMultiMapImplMut {
            multi_map: self,
            storage,
        }
    }

    fn commit(&mut self, storage: &mut S, id: u64) -> Result<(), DbError> {
        storage.commit(id)
    }

    fn contains(&self, storage: &S, key: &K) -> Result<bool, DbError> {
        if self.capacity() == 0 {
            return Ok(false);
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            match self.state(storage, pos)? {
                MapValueState::Empty => return Ok(false),
                MapValueState::Valid if self.key_at(storage, pos)? == *key => return Ok(true),
                MapValueState::Valid | MapValueState::Deleted => pos = self.next_pos(pos),
            }
        }
    }

    fn contains_value(&self, storage: &S, key: &K, value: &T) -> Result<bool, DbError> {
        if self.capacity() == 0 {
            return Ok(false);
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            match self.state(storage, pos)? {
                MapValueState::Empty => return Ok(false),
                MapValueState::Valid
                    if self.key_at(storage, pos)? == *key
                        && self.value_at(storage, pos)? == *value =>
                {
                    return Ok(true)
                }
                MapValueState::Valid | MapValueState::Deleted => pos = self.next_pos(pos),
            }
        }
    }

    fn insert(&mut self, storage: &mut S, key: &K, value: &T) -> Result<(), DbError> {
        let id = self.transaction(storage);
        let index = self.free_index(storage, key)?;
        self.do_insert(storage, index, key, value)?;
        self.commit(storage, id)
    }

    fn insert_or_replace<P: Fn(&T) -> bool>(
        &mut self,
        storage: &mut S,
        key: &K,
        predicate: P,
        new_value: &T,
    ) -> Result<Option<T>, DbError> {
        let id = self.transaction(storage);

        if self.len() >= self.max_len() {
            self.rehash(storage, self.capacity() * 2)?;
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let mut ret = None;

        loop {
            match self.state(storage, pos)? {
                MapValueState::Empty => {
                    self.do_insert(storage, pos, key, new_value)?;
                    break;
                }
                MapValueState::Valid if self.key_at(storage, pos)? == *key => {
                    let old_value = self.value_at(storage, pos)?;
                    if predicate(&old_value) {
                        self.set_value(storage, pos, new_value)?;
                        ret = Some(old_value);
                        break;
                    } else {
                        pos = self.next_pos(pos)
                    }
                }
                MapValueState::Valid | MapValueState::Deleted => pos = self.next_pos(pos),
            }
        }

        self.commit(storage, id)?;
        Ok(ret)
    }

    fn iter<'a>(&'a self, storage: &'a S) -> MapIterator<'a, K, T, S> {
        MapIterator {
            pos: 0,
            multi_map: self,
            storage,
        }
    }

    fn iter_key<'a>(&'a self, storage: &'a S, key: &'a K) -> MultiMapIterator<'a, K, T, S> {
        let pos = if self.capacity() == 0 {
            0
        } else {
            key.stable_hash() % self.capacity()
        };

        MultiMapIterator {
            pos,
            key,
            multi_map: self,
            storage,
        }
    }

    pub(crate) fn key_at(&self, storage: &S, index: u64) -> Result<K, DbError> {
        self.keys.read(storage).value(index)
    }

    fn remove_key(&mut self, storage: &mut S, key: &K) -> Result<(), DbError> {
        if self.capacity() == 0 {
            return Ok(());
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let mut len = self.len();

        let id = self.transaction(storage);

        loop {
            match self.state(storage, pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid if self.key_at(storage, pos)? == *key => {
                    self.drop_value(storage, pos)?;
                    len -= 1;
                }
                MapValueState::Valid | MapValueState::Deleted => {}
            }

            pos = self.next_pos(pos);
        }

        if len != self.len() {
            self.set_len(storage, len)?;

            if self.len() <= self.min_len() {
                self.rehash(storage, self.capacity() / 2)?;
            }
        }

        self.commit(storage, id)
    }

    fn remove_value(&mut self, storage: &mut S, key: &K, value: &T) -> Result<(), DbError> {
        if self.capacity() == 0 {
            return Ok(());
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        let id = self.transaction(storage);

        loop {
            match self.state(storage, pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid
                    if self.key_at(storage, pos)? == *key
                        && self.value_at(storage, pos)? == *value =>
                {
                    self.remove_index(storage, pos)?;
                    break;
                }
                MapValueState::Valid | MapValueState::Deleted => pos = self.next_pos(pos),
            }
        }

        self.commit(storage, id)
    }

    fn reserve(&mut self, storage: &mut S, capacity: u64) -> Result<(), DbError> {
        if self.capacity() < capacity {
            self.rehash(storage, capacity)?;
        }

        Ok(())
    }

    fn resize(&mut self, storage: &mut S, capacity: u64) -> Result<(), DbError> {
        self.states
            .write(storage)
            .resize(capacity, &MapValueState::Empty)?;
        self.keys.write(storage).resize(capacity, &K::default())?;
        self.values.write(storage).resize(capacity, &T::default())
    }

    fn set_len(&mut self, storage: &mut S, len: u64) -> Result<(), DbError> {
        self.data_index.len = len;
        storage.insert_at(self.storage_index, 0, &self.len())
    }

    fn set_state(
        &mut self,
        storage: &mut S,
        index: u64,
        state: MapValueState,
    ) -> Result<(), DbError> {
        self.states.write(storage).replace(index, &state)?;
        Ok(())
    }

    fn set_key(&mut self, storage: &mut S, index: u64, key: &K) -> Result<(), DbError> {
        self.keys.write(storage).replace(index, key)?;
        Ok(())
    }

    fn set_value(&mut self, storage: &mut S, index: u64, value: &T) -> Result<(), DbError> {
        self.values.write(storage).replace(index, value)?;
        Ok(())
    }

    pub(crate) fn state(&self, storage: &S, index: u64) -> Result<MapValueState, DbError> {
        self.states.read(storage).value(index)
    }

    fn swap(&mut self, storage: &mut S, index: u64, other: u64) -> Result<(), DbError> {
        self.states.write(storage).swap(index, other)?;
        self.keys.write(storage).swap(index, other)?;
        self.values.write(storage).swap(index, other)
    }

    fn transaction(&mut self, storage: &mut S) -> u64 {
        storage.transaction()
    }

    fn value(&self, storage: &S, key: &K) -> Result<Option<T>, DbError> {
        if self.capacity() == 0 {
            return Ok(None);
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            match self.state(storage, pos)? {
                MapValueState::Empty => {
                    return Ok(None);
                }
                MapValueState::Valid if self.key_at(storage, pos)? == *key => {
                    return Ok(Some(self.value_at(storage, pos)?));
                }
                MapValueState::Valid | MapValueState::Deleted => pos = self.next_pos(pos),
            }
        }
    }

    pub(crate) fn value_at(&self, storage: &S, index: u64) -> Result<T, DbError> {
        self.values.read(storage).value(index)
    }

    fn values(&self, storage: &S, key: &K) -> Result<Vec<T>, DbError> {
        if self.capacity() == 0 {
            return Ok(vec![]);
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let mut values = Vec::<T>::new();

        loop {
            match self.state(storage, pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid if self.key_at(storage, pos)? == *key => {
                    values.push(self.value_at(storage, pos)?)
                }
                MapValueState::Valid | MapValueState::Deleted => {}
            }

            pos = self.next_pos(pos)
        }

        Ok(values)
    }

    fn values_count(&self, storage: &S, key: &K) -> Result<u64, DbError> {
        if self.capacity() == 0 {
            return Ok(0);
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let mut result = 0;

        loop {
            match self.state(storage, pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid if self.key_at(storage, pos)? == *key => {
                    result += 1;
                }
                MapValueState::Valid | MapValueState::Deleted => {}
            }

            pos = self.next_pos(pos)
        }

        Ok(result)
    }

    fn do_insert(
        &mut self,
        storage: &mut S,
        index: u64,
        key: &K,
        value: &T,
    ) -> Result<(), DbError> {
        self.set_state(storage, index, MapValueState::Valid)?;
        self.set_key(storage, index, key)?;
        self.set_value(storage, index, value)?;
        self.set_len(storage, self.len() + 1)
    }

    fn drop_value(&mut self, storage: &mut S, pos: u64) -> Result<(), DbError> {
        self.set_state(storage, pos, MapValueState::Deleted)?;
        self.set_key(storage, pos, &K::default())?;
        self.set_value(storage, pos, &T::default())
    }

    fn free_index(&mut self, storage: &mut S, key: &K) -> Result<u64, DbError> {
        if self.len() >= self.max_len() {
            self.rehash(storage, self.capacity() * 2)?;
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();

        loop {
            match self.state(storage, pos)? {
                MapValueState::Empty | MapValueState::Deleted => break,
                MapValueState::Valid => pos = self.next_pos(pos),
            }
        }

        Ok(pos)
    }

    fn grow(
        &mut self,
        storage: &mut S,
        current_capacity: u64,
        new_capacity: u64,
    ) -> Result<(), DbError> {
        self.resize(storage, new_capacity)?;
        self.rehash_values(storage, current_capacity, new_capacity)
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

    fn rehash(&mut self, storage: &mut S, capacity: u64) -> Result<(), DbError> {
        let current_capacity = self.capacity();
        let new_capacity = std::cmp::max(capacity, 64_u64);

        match current_capacity.cmp(&new_capacity) {
            std::cmp::Ordering::Less => self.grow(storage, current_capacity, new_capacity),
            std::cmp::Ordering::Equal => Ok(()),
            std::cmp::Ordering::Greater => self.shrink(storage, current_capacity, new_capacity),
        }
    }

    fn rehash_deleted(
        &mut self,
        storage: &mut S,
        i: &mut u64,
        new_capacity: u64,
    ) -> Result<(), DbError> {
        if *i < new_capacity {
            self.set_state(storage, *i, MapValueState::Empty)?;
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
        storage: &mut S,
        i: &mut u64,
        new_capacity: u64,
        empty_list: &mut [bool],
    ) -> Result<(), DbError> {
        let key = self.key_at(storage, *i)?;
        let mut pos = key.stable_hash() % new_capacity;

        loop {
            if empty_list[pos as usize] {
                empty_list[pos as usize] = false;
                self.swap(storage, *i, pos)?;

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
        storage: &mut S,
        state: MapValueState,
        i: &mut u64,
        new_capacity: u64,
        empty_list: &mut [bool],
    ) -> Result<(), DbError> {
        match state {
            MapValueState::Empty => self.rehash_empty(i),
            MapValueState::Deleted => self.rehash_deleted(storage, i, new_capacity),
            MapValueState::Valid => self.rehash_valid(storage, i, new_capacity, empty_list),
        }
    }

    #[rustfmt::skip]
    fn rehash_values(
        &mut self,
        storage: &mut S,
        current_capacity: u64,
        new_capacity: u64,
    ) -> Result<(), DbError> {
        let mut i = 0_u64;
        let mut empty_list = vec![true; new_capacity as usize];

        while i != current_capacity {
            self.rehash_value(storage, self.state(storage, i)?, &mut i, new_capacity, &mut empty_list)?;
        }

        Ok(())
    }

    fn remove_index(&mut self, storage: &mut S, index: u64) -> Result<(), DbError> {
        self.drop_value(storage, index)?;
        self.set_len(storage, self.len() - 1)?;

        if self.len() <= self.min_len() {
            self.rehash(storage, self.capacity() / 2)?;
        }

        Ok(())
    }

    fn shrink(
        &mut self,
        storage: &mut S,
        current_capacity: u64,
        new_capacity: u64,
    ) -> Result<(), DbError> {
        self.rehash_values(storage, current_capacity, new_capacity)?;
        self.resize(storage, new_capacity)
    }
}

impl<'a, K, T, S> Iterator for MultiMapIterator<'a, K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    type Item = (K, T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let current_pos: u64 = self.pos;
            self.pos =
                if self.multi_map.capacity() == 0 || self.pos == self.multi_map.capacity() - 1 {
                    0
                } else {
                    self.pos + 1
                };

            match self
                .multi_map
                .state(self.storage, current_pos)
                .unwrap_or_default()
            {
                MapValueState::Empty => break,
                MapValueState::Deleted => {}
                MapValueState::Valid => {
                    let key = self
                        .multi_map
                        .key_at(self.storage, current_pos)
                        .unwrap_or_default();

                    if key == *self.key {
                        let value = self
                            .multi_map
                            .value_at(self.storage, current_pos)
                            .unwrap_or_default();
                        return Some((key, value));
                    }
                }
            }
        }

        None
    }
}

impl<'a, K, T, S> DbMultiMapImpl<'a, K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.multi_map.capacity()
    }

    pub fn contains(&self, key: &K) -> Result<bool, DbError> {
        self.multi_map.contains(self.storage, key)
    }

    pub fn contains_value(&self, key: &K, value: &T) -> Result<bool, DbError> {
        self.multi_map.contains_value(self.storage, key, value)
    }

    pub fn is_empty(&self) -> bool {
        self.multi_map.len() == 0
    }

    pub fn iter(&self) -> MapIterator<'a, K, T, S> {
        self.multi_map.iter(self.storage)
    }

    pub fn iter_key(&'a self, key: &'a K) -> MultiMapIterator<'a, K, T, S> {
        self.multi_map.iter_key(self.storage, key)
    }

    pub fn len(&self) -> u64 {
        self.multi_map.len()
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        self.multi_map.value(self.storage, key)
    }

    pub fn values(&self, key: &K) -> Result<Vec<T>, DbError> {
        self.multi_map.values(self.storage, key)
    }

    pub fn values_count(&self, key: &K) -> Result<u64, DbError> {
        self.multi_map.values_count(self.storage, key)
    }
}

impl<'a, K, T, S> DbMultiMapImplMut<'a, K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.multi_map.capacity()
    }

    pub fn contains(&self, key: &K) -> Result<bool, DbError> {
        self.multi_map.contains(self.storage, key)
    }

    pub fn contains_value(&self, key: &K, value: &T) -> Result<bool, DbError> {
        self.multi_map.contains_value(self.storage, key, value)
    }

    pub fn is_empty(&self) -> bool {
        self.multi_map.len() == 0
    }

    pub fn iter(&'a self) -> MapIterator<'a, K, T, S> {
        self.multi_map.iter(self.storage)
    }

    pub fn iter_key(&'a self, key: &'a K) -> MultiMapIterator<'a, K, T, S> {
        self.multi_map.iter_key(self.storage, key)
    }

    pub fn insert(&mut self, key: &K, value: &T) -> Result<(), DbError> {
        self.multi_map.insert(&mut self.storage, key, value)
    }

    pub fn insert_or_replace<P: Fn(&T) -> bool>(
        &mut self,
        key: &K,
        predicate: P,
        new_value: &T,
    ) -> Result<Option<T>, DbError> {
        self.multi_map
            .insert_or_replace(&mut self.storage, key, predicate, new_value)
    }

    pub fn len(&self) -> u64 {
        self.multi_map.len()
    }

    pub fn remove_key(&mut self, key: &K) -> Result<(), DbError> {
        self.multi_map.remove_key(&mut self.storage, key)
    }

    pub fn remove_value(&mut self, key: &K, value: &T) -> Result<(), DbError> {
        self.multi_map.remove_value(&mut self.storage, key, value)
    }

    pub fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        self.multi_map.reserve(&mut self.storage, capacity)
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        self.multi_map.value(self.storage, key)
    }

    pub fn values(&self, key: &K) -> Result<Vec<T>, DbError> {
        self.multi_map.values(self.storage, key)
    }

    pub fn values_count(&self, key: &K) -> Result<u64, DbError> {
        self.multi_map.values_count(self.storage, key)
    }
}
