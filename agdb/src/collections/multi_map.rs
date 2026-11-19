use crate::DbError;
use crate::DbErrorType;
use crate::StorageData;
use crate::collections::map::DbMapData;
use crate::collections::map::MapData;
use crate::collections::map::MapIterator;
use crate::collections::map::MapValueState;
use crate::collections::map::OldSlot;
use crate::collections::vec::VecValue;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::stable_hash::StableHash;
use std::marker::PhantomData;

const U32_SLOTS: u64 = u32::MAX as u64;

fn no_free_slot() -> DbError {
    DbError::collections(
        DbErrorType::InvalidIndex,
        "map has more valid slots than it counts",
    )
}

pub struct MultiMapImpl<K, T, D, Data>
where
    D: StorageData,
    Data: MapData<K, T, D>,
{
    pub(crate) data: Data,
    pub(crate) phantom_marker: PhantomData<(K, T, D)>,
}

pub struct MultiMapIterator<'a, K, T, D, Data>
where
    D: StorageData,
    Data: MapData<K, T, D>,
{
    pub pos: u64,
    pub start: u64,
    pub key: &'a K,
    pub data: &'a Data,
    pub storage: &'a Storage<D>,
    pub phantom_data: PhantomData<(T, D)>,
}

impl<K, T, D, Data> Iterator for MultiMapIterator<'_, K, T, D, Data>
where
    K: Default + PartialEq,
    T: Default,
    D: StorageData,
    Data: MapData<K, T, D>,
{
    type Item = Result<(K, T), DbError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.capacity() == 0 {
            return None;
        }

        loop {
            let current_pos = self.pos;

            self.pos = if self.pos == self.data.capacity() - 1 {
                0
            } else {
                self.pos + 1
            };

            match self.data.state(self.storage, current_pos) {
                Err(error) => return Some(Err(error)),
                Ok(MapValueState::Empty) => break,
                Ok(MapValueState::Deleted) => {}
                Ok(MapValueState::Valid) => {
                    let key = match self.data.key(self.storage, current_pos) {
                        Ok(key) => key,
                        Err(error) => return Some(Err(error)),
                    };

                    if key == *self.key {
                        let value = match self.data.value(self.storage, current_pos) {
                            Ok(value) => value,
                            Err(error) => return Some(Err(error)),
                        };
                        return Some(Ok((key, value)));
                    }
                }
            }

            if self.start == self.pos {
                break;
            }
        }

        None
    }
}

impl<K, T, D, Data> MultiMapImpl<K, T, D, Data>
where
    K: Default + PartialEq + StableHash,
    T: Default + PartialEq,
    D: StorageData,
    Data: MapData<K, T, D>,
{
    pub fn capacity(&self) -> u64 {
        self.data.capacity()
    }

    pub fn contains(&self, storage: &Storage<D>, key: &K) -> Result<bool, DbError> {
        match self.iter_key(storage, key).next() {
            Some(result) => result.map(|_| true),
            None => Ok(false),
        }
    }

    #[allow(dead_code)]
    pub fn contains_value(
        &self,
        storage: &Storage<D>,
        key: &K,
        value: &T,
    ) -> Result<bool, DbError> {
        for result in self.iter_key(storage, key) {
            if result?.1 == *value {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn insert(&mut self, storage: &mut Storage<D>, key: &K, value: &T) -> Result<(), DbError> {
        let id = self.data.transaction(storage);
        let index = self.free_index(storage, key)?;
        self.do_insert(storage, index, key, value)?;
        self.data.commit(storage, id)
    }

    pub fn insert_or_replace<P: Fn(&T) -> bool>(
        &mut self,
        storage: &mut Storage<D>,
        key: &K,
        predicate: P,
        new_value: &T,
    ) -> Result<Option<T>, DbError> {
        let id = self.data.transaction(storage);

        self.reserve_slot(storage)?;

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let start_pos = pos;
        let mut free_pos = None;
        let mut ret = None;

        loop {
            match self.data.state(storage, pos)? {
                MapValueState::Empty => {
                    free_pos.get_or_insert(pos);
                    break;
                }
                MapValueState::Deleted => {
                    if free_pos.is_none() {
                        free_pos = Some(pos);
                    }
                }
                MapValueState::Valid if self.data.key(storage, pos)? == *key => {
                    let old_value = self.data.value(storage, pos)?;
                    if predicate(&old_value) {
                        self.data.set_value(storage, pos, new_value)?;
                        ret = Some(old_value);
                        free_pos = None;
                        break;
                    }
                }
                MapValueState::Valid => {}
            }

            pos = self.next_pos(pos);

            if pos == start_pos {
                return Err(no_free_slot());
            }
        }

        if let Some(pos) = free_pos {
            self.do_insert(storage, pos, key, new_value)?;
        }

        self.data.commit(storage, id)?;
        Ok(ret)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter<'a>(&'a self, storage: &'a Storage<D>) -> MapIterator<'a, K, T, D, Data> {
        MapIterator {
            pos: 0,
            data: &self.data,
            storage,
            phantom_data: PhantomData,
        }
    }

    pub fn iter_key<'a>(
        &'a self,
        storage: &'a Storage<D>,
        key: &'a K,
    ) -> MultiMapIterator<'a, K, T, D, Data> {
        let pos = if self.capacity() == 0 {
            0
        } else {
            key.stable_hash() % self.capacity()
        };

        MultiMapIterator {
            pos,
            start: pos,
            key,
            data: &self.data,
            storage,
            phantom_data: PhantomData,
        }
    }

    pub fn len(&self) -> u64 {
        self.data.len()
    }

    #[allow(dead_code)]
    pub fn remove_key(&mut self, storage: &mut Storage<D>, key: &K) -> Result<(), DbError> {
        self.remove_key_impl(storage, key, false)
    }

    pub fn remove_value(
        &mut self,
        storage: &mut Storage<D>,
        key: &K,
        value: &T,
    ) -> Result<(), DbError> {
        if self.capacity() == 0 {
            return Ok(());
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let start_pos = pos;
        let id = self.data.transaction(storage);

        loop {
            match self.data.state(storage, pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid
                    if self.data.key(storage, pos)? == *key
                        && self.data.value(storage, pos)? == *value =>
                {
                    self.remove_index(storage, pos)?;
                    break;
                }
                MapValueState::Valid | MapValueState::Deleted => pos = self.next_pos(pos),
            }

            if pos == start_pos {
                self.purge_deleted(storage)?;
                break;
            }
        }

        self.data.commit(storage, id)
    }

    pub fn remove_from_storage(self, storage: &mut Storage<D>) -> Result<(), DbError> {
        self.data.remove_from_storage(storage)
    }

    pub fn reserve(&mut self, storage: &mut Storage<D>, capacity: u64) -> Result<(), DbError> {
        let mut new_capacity = std::cmp::max(self.capacity(), 64);

        while new_capacity < capacity {
            new_capacity *= 2;
        }

        self.rehash(storage, new_capacity)
    }

    pub fn shrink_to_fit(&mut self, storage: &mut Storage<D>) -> Result<(), DbError> {
        self.data.shrink_to_fit(storage)
    }

    pub fn value(&self, storage: &Storage<D>, key: &K) -> Result<Option<T>, DbError> {
        match self.iter_key(storage, key).next() {
            Some(result) => result.map(|(_, val)| Some(val)),
            None => Ok(None),
        }
    }

    pub fn values(&self, storage: &Storage<D>, key: &K) -> Result<Vec<T>, DbError> {
        self.iter_key(storage, key)
            .map(|result| result.map(|(_, value)| value))
            .collect()
    }

    #[allow(dead_code)]
    pub fn values_count(&self, storage: &Storage<D>, key: &K) -> Result<u64, DbError> {
        self.iter_key(storage, key)
            .try_fold(0_u64, |count, result| result.map(|_| count + 1))
    }

    fn do_insert(
        &mut self,
        storage: &mut Storage<D>,
        index: u64,
        key: &K,
        value: &T,
    ) -> Result<(), DbError> {
        self.data.set_key(storage, index, key)?;
        self.data.set_value(storage, index, value)?;
        self.data.set_state(storage, index, MapValueState::Valid)
    }

    fn drop_value(&mut self, storage: &mut Storage<D>, pos: u64) -> Result<(), DbError> {
        self.data.set_state(storage, pos, MapValueState::Deleted)?;
        self.data.set_key(storage, pos, &K::default())?;
        self.data.set_value(storage, pos, &T::default())
    }

    fn free_index(&mut self, storage: &mut Storage<D>, key: &K) -> Result<u64, DbError> {
        self.reserve_slot(storage)?;

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let start_pos = pos;

        loop {
            match self.data.state(storage, pos)? {
                MapValueState::Empty | MapValueState::Deleted => break,
                MapValueState::Valid => pos = self.next_pos(pos),
            }

            if pos == start_pos {
                return Err(no_free_slot());
            }
        }

        Ok(pos)
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

    fn purge_deleted(&mut self, storage: &mut Storage<D>) -> Result<(), DbError> {
        self.rebuild(storage, self.capacity(), U32_SLOTS)
    }

    fn rehash(&mut self, storage: &mut Storage<D>, capacity: u64) -> Result<(), DbError> {
        let new_capacity = std::cmp::max(capacity, 64_u64);

        if new_capacity == self.capacity() {
            return Ok(());
        }

        self.rebuild(storage, new_capacity, U32_SLOTS)
    }

    fn rebuild(
        &mut self,
        storage: &mut Storage<D>,
        new_capacity: u64,
        u32_slots: u64,
    ) -> Result<(), DbError> {
        if self.capacity() <= u32_slots {
            self.rebuild_with::<u32>(storage, new_capacity)
        } else {
            self.rebuild_with::<u64>(storage, new_capacity)
        }
    }

    fn rebuild_with<S: OldSlot>(
        &mut self,
        storage: &mut Storage<D>,
        new_capacity: u64,
    ) -> Result<(), DbError> {
        let mut old_slots = vec![S::none(); new_capacity as usize];
        let mut placed = 0;

        self.data.for_each_valid_key(storage, |slot, key| {
            if placed == old_slots.len() {
                return Err(no_free_slot());
            }

            placed += 1;
            let mut pos = (key.stable_hash() % new_capacity) as usize;

            while old_slots[pos] != S::none() {
                pos += 1;

                if pos == old_slots.len() {
                    pos = 0;
                }
            }

            old_slots[pos] = S::new(slot);
            Ok(())
        })?;

        self.data.rebuild(storage, &old_slots)
    }

    fn remove_index(&mut self, storage: &mut Storage<D>, index: u64) -> Result<(), DbError> {
        self.drop_value(storage, index)?;

        if self.len() <= self.min_len() {
            self.rehash(storage, self.capacity() / 2)?;
        }

        Ok(())
    }

    pub(super) fn remove_key_impl(
        &mut self,
        storage: &mut Storage<D>,
        key: &K,
        unique: bool,
    ) -> Result<(), DbError> {
        if self.capacity() == 0 {
            return Ok(());
        }

        let hash = key.stable_hash();
        let mut pos = hash % self.capacity();
        let start_pos = pos;
        let len = self.len();

        let id = self.data.transaction(storage);

        loop {
            match self.data.state(storage, pos)? {
                MapValueState::Empty => break,
                MapValueState::Valid if self.data.key(storage, pos)? == *key => {
                    self.drop_value(storage, pos)?;

                    if unique {
                        break;
                    }
                }
                MapValueState::Valid | MapValueState::Deleted => {}
            }

            pos = self.next_pos(pos);

            if pos == start_pos {
                self.purge_deleted(storage)?;
                break;
            }
        }

        if self.len() != len && self.len() <= self.min_len() {
            self.rehash(storage, self.capacity() / 2)?;
        }

        self.data.commit(storage, id)
    }

    fn reserve_slot(&mut self, storage: &mut Storage<D>) -> Result<(), DbError> {
        let deleted = self.data.deleted();

        if self.len() + deleted < self.max_len() {
            return Ok(());
        }

        if self.len() < self.max_len() && self.capacity() / 16 < deleted {
            self.purge_deleted(storage)
        } else {
            self.rehash(storage, self.capacity() * 2)
        }
    }
}

pub type MultiMapStorage<K, T, D> = MultiMapImpl<K, T, D, DbMapData<K, T, D>>;

impl<K, T, D> MultiMapStorage<K, T, D>
where
    K: Clone + Default + PartialEq + VecValue<D>,
    T: Clone + Default + PartialEq + VecValue<D>,
    D: StorageData,
{
    pub fn new(storage: &mut Storage<D>) -> Result<Self, DbError> {
        Ok(Self {
            data: DbMapData::<K, T, D>::new(storage)?,
            phantom_marker: PhantomData,
        })
    }

    pub fn from_storage(storage: &Storage<D>, index: StorageIndex) -> Result<Self, DbError> {
        Ok(Self {
            data: DbMapData::<K, T, D>::from_storage(storage, index)?,
            phantom_marker: PhantomData,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.data.storage_index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::map::MapDataIndex;
    use crate::collections::tests::TestStorage;
    use crate::collections::tests::assert_io_failures;
    use crate::collections::tests::assert_io_failures_keep;
    use crate::collections::tests::assert_io_failures_recover;
    use crate::collections::tests::storage;
    use crate::storage::file_storage::FileStorageImpl;
    use agplatform::test_platform::TestPlatform;

    type TestMap = MultiMapStorage<u64, u64, FileStorageImpl<TestPlatform>>;

    fn map(storage: &mut TestStorage, pairs: &[(u64, u64)]) -> TestMap {
        let mut map = TestMap::new(storage).unwrap();

        for (key, value) in pairs {
            map.insert(storage, key, value).unwrap();
        }

        map
    }

    fn filled(storage: &mut TestStorage, keys: std::ops::Range<u64>) -> TestMap {
        map(storage, &keys.map(|i| (i, i)).collect::<Vec<_>>())
    }

    /// Keys 5..64 in their own slots of a 64 slot map with the first five
    /// slots deleted and none empty, as maps written before empty slots were
    /// reserved can be.
    fn saturated(storage: &mut TestStorage) -> TestMap {
        let mut map = filled(storage, 5..64);

        for i in 0..5 {
            map.data
                .set_state(storage, i, MapValueState::Deleted)
                .unwrap();
        }

        assert_eq!(state_count(&map, storage, MapValueState::Empty), 0);
        map
    }

    /// Writes the state of a slot behind the map's back, as damage on disk
    /// would.
    fn write_state(storage: &mut TestStorage, map: &TestMap, slot: u64, state: MapValueState) {
        let index = storage
            .value::<MapDataIndex>(map.data.storage_index())
            .unwrap();
        storage
            .insert_at(index.states_index, 8 + slot, &state)
            .unwrap();
    }

    /// A 64 slot map of key 1 whose other slots turned valid behind its back.
    fn uncounted(storage: &mut TestStorage) -> TestMap {
        let map = filled(storage, 1..2);

        for slot in 0..64 {
            write_state(storage, &map, slot, MapValueState::Valid);
        }

        map
    }

    /// A 128 slot map that halves on the next removal, with keys that move
    /// when it does.
    fn shrinking(storage: &mut TestStorage) -> TestMap {
        let mut map = map(storage, &[]);
        map.reserve(storage, 128).unwrap();

        for i in 64..121 {
            map.insert(storage, &i, &i).unwrap();
        }

        map
    }

    fn state_count(map: &TestMap, storage: &TestStorage, state: MapValueState) -> u64 {
        (0..map.capacity())
            .filter(|i| map.data.state(storage, *i).unwrap() == state)
            .count() as u64
    }

    fn assert_purged(map: &TestMap, storage: &TestStorage) {
        assert_eq!(map.capacity(), 64);
        assert_eq!(map.data.deleted(), 0);
        assert_eq!(state_count(map, storage, MapValueState::Deleted), 0);
        assert_eq!(state_count(map, storage, MapValueState::Valid), map.len());

        for i in 5..64 {
            assert_eq!(map.value(storage, &i), Ok(Some(i)));
        }
    }

    /// Predicates as one function pointer type, so that all tests share an
    /// instantiation of `insert_or_replace`.
    fn when(predicate: fn(&u64) -> bool) -> fn(&u64) -> bool {
        predicate
    }

    fn sorted(map: &TestMap, storage: &TestStorage) -> Vec<(u64, u64)> {
        let mut pairs = map.iter(storage).collect::<Vec<_>>();
        pairs.sort();
        pairs
    }

    mod multi_map_iterator {
        use super::*;

        mod iterator {
            use super::*;

            mod next {
                use super::*;

                #[test]
                fn next_should_yield_values_of_key_only() {
                    let mut storage = storage();
                    let mut map = map(&mut storage, &[(3, 30), (1, 10), (1, 20), (1, 30), (4, 40)]);
                    map.remove_value(&mut storage, &1, &10).unwrap();

                    let mut values = map
                        .iter_key(&storage, &1)
                        .collect::<Result<Vec<_>, DbError>>()
                        .unwrap();

                    values.sort();
                    assert_eq!(values, vec![(1, 20), (1, 30)]);
                }

                #[test]
                fn next_of_new_map_should_be_none() {
                    let mut storage = storage();
                    let map = map(&mut storage, &[]);

                    assert!(map.iter_key(&storage, &1).next().is_none());
                }

                #[test]
                fn next_without_empty_slots_should_stop_after_wrapping() {
                    let mut storage = storage();
                    let map = saturated(&mut storage);

                    assert!(map.iter_key(&storage, &100).next().is_none());
                }

                #[test]
                fn next_failure_should_fail() {
                    assert_io_failures(
                        |storage| map(storage, &[(1, 10), (65, 20)]),
                        |storage, map| map.values(storage, &65),
                    );
                }
            }
        }
    }

    mod multi_map_impl {
        use super::*;

        #[test]
        fn multi_map_impl_with_reused_deleted_slots_should_keep_finding_values() {
            let mut storage = storage();
            let mut map = filled(&mut storage, 0..60);

            for i in 0..4 {
                map.remove_key(&mut storage, &i).unwrap();
            }
            for i in 60..64 {
                map.insert(&mut storage, &i, &i).unwrap();
            }
            map.remove_key(&mut storage, &32).unwrap();
            map.remove_value(&mut storage, &33, &33).unwrap();

            assert_eq!(map.len(), 58);
            assert_eq!(map.value(&storage, &0), Ok(None));
            assert_eq!(map.value(&storage, &32), Ok(None));
            assert_eq!(map.value(&storage, &33), Ok(None));
            for i in (4..32).chain(34..64) {
                assert_eq!(map.value(&storage, &i), Ok(Some(i)));
            }
        }

        #[test]
        fn multi_map_impl_under_churn_should_keep_empty_slots() {
            // GIVEN a map filled close to its maximum length
            let mut storage = storage();
            let mut map = filled(&mut storage, 0..200);

            // WHEN keys are replaced by new ones at the same length
            for i in 0..2000 {
                map.remove_key(&mut storage, &i).unwrap();
                map.insert(&mut storage, &(i + 200), &i).unwrap();
            }

            // THEN lookups still find empty slots to stop at
            assert_eq!(map.len(), 200);
            assert_eq!(map.capacity(), 256);
            assert!(state_count(&map, &storage, MapValueState::Empty) > 0);
            assert_eq!(map.value(&storage, &1_000_000), Ok(None));
        }

        mod capacity {
            use super::*;

            #[test]
            fn capacity_should_start_at_zero_then_minimum() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[]);
                assert_eq!(map.capacity(), 0);

                map.insert(&mut storage, &1, &1).unwrap();

                assert_eq!(map.capacity(), 64);
            }
        }

        mod contains {
            use super::*;

            #[test]
            fn contains_should_find_key_with_any_value() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 10), (1, 20)]);
                map.remove_value(&mut storage, &1, &10).unwrap();

                assert_eq!(map.contains(&storage, &1), Ok(true));
                assert_eq!(map.contains(&storage, &2), Ok(false));
            }

            #[test]
            fn contains_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[(1, 10)]),
                    |storage, map| map.contains(storage, &1),
                );
            }
        }

        mod contains_value {
            use super::*;

            #[test]
            fn contains_value_should_match_any_value_of_key() {
                let mut storage = storage();
                let map = map(&mut storage, &[(1, 10), (1, 20), (2, 30)]);

                assert_eq!(map.contains_value(&storage, &1, &20), Ok(true));
                assert_eq!(map.contains_value(&storage, &1, &30), Ok(false));
                assert_eq!(map.contains_value(&storage, &3, &30), Ok(false));
            }

            #[test]
            fn contains_value_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[(1, 10), (1, 20)]),
                    |storage, map| map.contains_value(storage, &1, &20),
                );
            }
        }

        mod insert {
            use super::*;

            #[test]
            fn insert_should_keep_all_values_of_key() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[]);

                map.insert(&mut storage, &1, &10).unwrap();
                map.insert(&mut storage, &1, &20).unwrap();
                map.insert(&mut storage, &1, &10).unwrap();

                assert_eq!(map.len(), 3);
                assert_eq!(sorted(&map, &storage), vec![(1, 10), (1, 10), (1, 20)]);
            }

            #[test]
            fn insert_into_saturated_map_should_purge_deleted() {
                let mut storage = storage();
                let mut map = saturated(&mut storage);

                map.insert(&mut storage, &100, &100).unwrap();

                assert_eq!(map.len(), 60);
                assert_eq!(map.value(&storage, &100), Ok(Some(100)));
                assert_purged(&map, &storage);
            }

            #[test]
            fn insert_without_free_slots_should_fail() {
                let mut storage = storage();
                let mut map = uncounted(&mut storage);

                assert_eq!(map.insert(&mut storage, &70, &70), Err(no_free_slot()));
            }

            #[test]
            fn insert_failure_should_fail() {
                assert_io_failures(
                    |storage| filled(storage, 0..60),
                    |storage, map| map.insert(storage, &60, &60),
                );
            }
        }

        mod insert_or_replace {
            use super::*;

            #[test]
            fn insert_or_replace_matching_value_should_replace_it() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 10), (1, 20)]);

                assert_eq!(
                    map.insert_or_replace(&mut storage, &1, when(|v| *v == 20), &30),
                    Ok(Some(20))
                );

                assert_eq!(map.len(), 2);
                assert_eq!(sorted(&map, &storage), vec![(1, 10), (1, 30)]);
            }

            #[test]
            fn insert_or_replace_without_match_should_insert() {
                let mut storage = storage();
                let mut empty = map(&mut storage, &[]);
                let mut map = map(&mut storage, &[(1, 10), (2, 20)]);

                assert_eq!(
                    empty.insert_or_replace(&mut storage, &1, when(|_| true), &10),
                    Ok(None)
                );
                assert_eq!(
                    map.insert_or_replace(&mut storage, &1, when(|v| *v == 20), &30),
                    Ok(None)
                );

                assert_eq!(sorted(&empty, &storage), vec![(1, 10)]);
                assert_eq!(sorted(&map, &storage), vec![(1, 10), (1, 30), (2, 20)]);
            }

            #[test]
            fn insert_or_replace_should_reuse_first_deleted_slot() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 10), (1, 20), (1, 21), (1, 30)]);
                map.remove_value(&mut storage, &1, &20).unwrap();
                map.remove_value(&mut storage, &1, &21).unwrap();

                map.insert_or_replace(&mut storage, &1, when(|_| false), &40)
                    .unwrap();

                assert_eq!(map.data.value(&storage, 2), Ok(40));
                assert_eq!(map.data.state(&storage, 3), Ok(MapValueState::Deleted));
                assert_eq!(map.values(&storage, &1), Ok(vec![10, 40, 30]));
            }

            #[test]
            fn insert_or_replace_into_saturated_map_should_purge_deleted() {
                let mut storage = storage();
                let mut map = saturated(&mut storage);

                map.insert_or_replace(&mut storage, &100, when(|_| false), &100)
                    .unwrap();

                assert_eq!(map.len(), 60);
                assert_eq!(map.value(&storage, &100), Ok(Some(100)));
                assert_purged(&map, &storage);
            }

            #[test]
            fn insert_or_replace_without_free_slots_should_fail() {
                let mut storage = storage();
                let mut map = uncounted(&mut storage);

                assert_eq!(
                    map.insert_or_replace(&mut storage, &70, when(|_| false), &70),
                    Err(no_free_slot())
                );
            }

            #[test]
            fn insert_or_replace_failure_should_fail() {
                assert_io_failures(
                    |storage| filled(storage, 0..60),
                    |storage, map| map.insert_or_replace(storage, &60, when(|_| false), &60),
                );
                assert_io_failures(
                    |storage| filled(storage, 0..2),
                    |storage, map| map.insert_or_replace(storage, &1, when(|_| true), &2),
                );
            }
        }

        mod is_empty {
            use super::*;

            #[test]
            fn is_empty_should_follow_len() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[]);
                assert!(map.is_empty());

                map.insert(&mut storage, &1, &10).unwrap();
                assert!(!map.is_empty());
            }
        }

        mod iter {
            use super::*;

            #[test]
            fn iter_should_yield_every_value_of_every_key() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[]);
                for i in 0..100 {
                    map.insert(&mut storage, &i, &i).unwrap();
                    map.insert(&mut storage, &i, &(i + 1)).unwrap();
                }
                map.remove_value(&mut storage, &10, &10).unwrap();

                let expected = (0..100)
                    .flat_map(|i| [(i, i), (i, i + 1)])
                    .filter(|pair| *pair != (10, 10))
                    .collect::<Vec<_>>();
                assert_eq!(sorted(&map, &storage), expected);
            }
        }

        mod iter_key {
            use super::*;

            #[test]
            fn iter_key_should_start_at_hash_of_key() {
                let mut storage = storage();
                let map = map(&mut storage, &[(1, 10)]);

                let iter = map.iter_key(&storage, &65);

                assert_eq!(iter.pos, 1);
                assert_eq!(iter.start, 1);
            }
        }

        mod len {
            use super::*;

            #[test]
            fn len_should_count_values() {
                let mut storage = storage();

                assert_eq!(map(&mut storage, &[(1, 10), (1, 11), (2, 20)]).len(), 3);
            }
        }

        mod remove_key {
            use super::*;

            #[test]
            fn remove_key_should_remove_all_values_of_key() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 10), (2, 20), (1, 11)]);

                map.remove_key(&mut storage, &1).unwrap();

                assert_eq!(map.len(), 1);
                assert_eq!(sorted(&map, &storage), vec![(2, 20)]);
            }

            #[test]
            fn remove_key_missing_should_do_nothing() {
                let mut storage = storage();
                let mut empty = map(&mut storage, &[]);
                let mut map = map(&mut storage, &[(1, 10)]);

                empty.remove_key(&mut storage, &1).unwrap();
                map.remove_key(&mut storage, &2).unwrap();

                assert_eq!(sorted(&map, &storage), vec![(1, 10)]);
            }

            #[test]
            fn remove_key_missing_from_saturated_map_should_purge_deleted() {
                let mut storage = storage();
                let mut map = saturated(&mut storage);

                map.remove_key(&mut storage, &100).unwrap();

                assert_eq!(map.len(), 59);
                assert_purged(&map, &storage);
            }

            #[test]
            fn remove_key_under_min_len_should_halve_capacity() {
                let mut storage = storage();
                let mut map = filled(&mut storage, 0..100);

                for i in (0..100).rev() {
                    map.remove_key(&mut storage, &i).unwrap();
                }

                assert_eq!(map.len(), 0);
                assert_eq!(map.capacity(), 64);
            }

            #[test]
            fn remove_key_failure_should_fail() {
                assert_io_failures(shrinking, |storage, map| map.remove_key(storage, &100));
                assert_io_failures(saturated, |storage, map| map.remove_key(storage, &100));
            }
        }

        mod remove_value {
            use super::*;

            #[test]
            fn remove_value_should_remove_only_matching_value() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 10), (1, 20), (2, 10)]);

                map.remove_value(&mut storage, &1, &10).unwrap();

                assert_eq!(map.len(), 2);
                assert_eq!(sorted(&map, &storage), vec![(1, 20), (2, 10)]);
            }

            #[test]
            fn remove_value_missing_should_do_nothing() {
                let mut storage = storage();
                let mut empty = map(&mut storage, &[]);
                let mut map = map(&mut storage, &[(1, 10)]);

                empty.remove_value(&mut storage, &1, &10).unwrap();
                map.remove_value(&mut storage, &1, &20).unwrap();
                map.remove_value(&mut storage, &2, &10).unwrap();

                assert_eq!(sorted(&map, &storage), vec![(1, 10)]);
            }

            #[test]
            fn remove_value_missing_from_saturated_map_should_purge_deleted() {
                let mut storage = storage();
                let mut map = saturated(&mut storage);

                map.remove_value(&mut storage, &100, &100).unwrap();

                assert_eq!(map.len(), 59);
                assert_purged(&map, &storage);
            }

            #[test]
            fn remove_value_under_min_len_should_halve_capacity() {
                let mut storage = storage();
                let mut map = filled(&mut storage, 0..100);

                for i in (0..100).rev() {
                    map.remove_value(&mut storage, &i, &i).unwrap();
                }

                assert_eq!(map.len(), 0);
                assert_eq!(map.capacity(), 64);
            }

            #[test]
            fn remove_value_failure_should_fail() {
                assert_io_failures(shrinking, |storage, map| {
                    map.remove_value(storage, &100, &100)
                });
                assert_io_failures(saturated, |storage, map| {
                    map.remove_value(storage, &100, &100)
                });
            }
        }

        mod remove_from_storage {
            use super::*;

            #[test]
            fn remove_from_storage_should_remove_map_and_its_values() {
                let mut storage = storage();
                let len = storage.len();
                let map = filled(&mut storage, 0..100);

                map.remove_from_storage(&mut storage).unwrap();

                storage.optimize_storage().unwrap();
                assert_eq!(storage.len(), len);
            }
        }

        mod reserve {
            use super::*;

            #[test]
            fn reserve_larger_should_double_capacity_and_rehash_values() {
                let mut storage = storage();
                let mut map = filled(&mut storage, 0..10);

                map.reserve(&mut storage, 200).unwrap();

                assert_eq!(map.capacity(), 256);
                assert_eq!(
                    sorted(&map, &storage),
                    (0..10).map(|i| (i, i)).collect::<Vec<_>>()
                );
            }

            #[test]
            fn reserve_smaller_should_keep_capacity() {
                let mut storage = storage();
                let mut map = filled(&mut storage, 0..10);

                map.reserve(&mut storage, 10).unwrap();

                assert_eq!(map.capacity(), 64);
            }

            #[test]
            fn reserve_failure_should_fail() {
                assert_io_failures(
                    |storage| filled(storage, 0..10),
                    |storage, map| map.reserve(storage, 200),
                );
            }
        }

        mod shrink_to_fit {
            use super::*;

            #[test]
            fn shrink_to_fit_should_keep_capacity_and_values() {
                let mut storage = storage();
                let mut map = filled(&mut storage, 0..100);
                for i in 1..100 {
                    map.remove_key(&mut storage, &i).unwrap();
                }

                map.shrink_to_fit(&mut storage).unwrap();

                assert_eq!(map.capacity(), 64);
                assert_eq!(sorted(&map, &storage), vec![(0, 0)]);
            }
        }

        mod value {
            use super::*;

            #[test]
            fn value_should_return_a_value_of_key() {
                let mut storage = storage();
                let map = map(&mut storage, &[(1, 10), (2, 20)]);

                assert_eq!(map.value(&storage, &2), Ok(Some(20)));
                assert_eq!(map.value(&storage, &3), Ok(None));
            }

            #[test]
            fn value_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[(1, 10)]),
                    |storage, map| map.value(storage, &1),
                );
            }
        }

        mod values {
            use super::*;

            #[test]
            fn values_should_return_all_values_of_key() {
                let mut storage = storage();
                let map = map(&mut storage, &[(1, 10), (2, 20), (1, 11)]);

                assert_eq!(map.values(&storage, &1), Ok(vec![10, 11]));
                assert_eq!(map.values(&storage, &3), Ok(vec![]));
            }
        }

        mod values_count {
            use super::*;

            #[test]
            fn values_count_should_count_values_of_key() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 10), (1, 11), (1, 12), (2, 20)]);
                map.remove_value(&mut storage, &1, &11).unwrap();

                assert_eq!(map.values_count(&storage, &1), Ok(2));
                assert_eq!(map.values_count(&storage, &2), Ok(1));
                assert_eq!(map.values_count(&storage, &3), Ok(0));
            }

            #[test]
            fn values_count_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[(1, 10)]),
                    |storage, map| map.values_count(storage, &1),
                );
            }
        }

        mod rebuild {
            use super::*;

            type StringMap = MultiMapStorage<String, String, FileStorageImpl<TestPlatform>>;

            #[test]
            fn rebuild_with_wide_table_should_keep_values() {
                let mut storage = storage();
                let mut map = filled(&mut storage, 0..50);
                map.remove_key(&mut storage, &7).unwrap();

                map.rebuild(&mut storage, 128, 0).unwrap();

                assert_eq!(map.capacity(), 128);
                assert_eq!(map.data.deleted(), 0);
                assert_eq!(
                    sorted(&map, &storage),
                    (0..50)
                        .filter(|i| *i != 7)
                        .map(|i| (i, i))
                        .collect::<Vec<_>>()
                );
            }

            #[test]
            fn rebuild_with_keys_hashing_to_last_slot_should_wrap_around() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(63, 1), (127, 2), (191, 3)]);

                map.rebuild(&mut storage, 64, U32_SLOTS).unwrap();

                assert_eq!(map.data.key(&storage, 63), Ok(127));
                assert_eq!(map.data.key(&storage, 0), Ok(191));
                assert_eq!(map.data.key(&storage, 1), Ok(63));
                assert_eq!(map.value(&storage, &191), Ok(Some(3)));
            }

            #[test]
            fn rebuild_with_more_valid_slots_than_capacity_should_fail() {
                let mut storage = storage();
                let mut map = StringMap::new(&mut storage).unwrap();
                map.reserve(&mut storage, 128).unwrap();
                for i in 0..128 {
                    map.data
                        .set_state(&mut storage, i, MapValueState::Valid)
                        .unwrap();
                }

                assert_eq!(
                    map.rebuild(&mut storage, 64, U32_SLOTS),
                    Err(no_free_slot())
                );
                assert_eq!(map.capacity(), 128);
            }

            #[test]
            fn rebuild_should_free_records_no_slot_takes_over() {
                let mut storage = storage();
                let len = storage.len();
                let mut map = StringMap::new(&mut storage).unwrap();

                for round in 0..3 {
                    for i in 0..200 {
                        map.insert(
                            &mut storage,
                            &format!("key {i} of a long round {round}"),
                            &format!("value {i}"),
                        )
                        .unwrap();
                    }

                    for i in 0..195 {
                        map.remove_key(&mut storage, &format!("key {i} of a long round {round}"))
                            .unwrap();
                    }
                }

                map.remove_from_storage(&mut storage).unwrap();
                storage.optimize_storage().unwrap();

                assert_eq!(storage.len(), len);
            }

            #[test]
            fn rebuild_failure_should_be_recovered() {
                assert_io_failures_recover(
                    |storage| {
                        let mut map = filled(storage, 0..40);
                        map.remove_key(storage, &3).unwrap();
                        let pairs = sorted(&map, storage);
                        (map, pairs)
                    },
                    |storage, (map, _)| map.rebuild(storage, 128, U32_SLOTS),
                    |storage, (map, pairs)| {
                        let map = TestMap::from_storage(storage, map.data.storage_index()).unwrap();
                        assert_eq!(map.capacity(), 64);
                        assert_eq!(&sorted(&map, storage), pairs);
                    },
                );
            }

            #[test]
            fn rebuild_of_string_map_failure_should_be_recovered() {
                for (capacity, keys, removed, new_capacity) in STRING_CASES {
                    assert_io_failures_recover(
                        |storage| string_map(storage, capacity, keys.clone(), removed.clone()),
                        |storage, (map, _)| map.rebuild(storage, new_capacity, U32_SLOTS),
                        |storage, (map, pairs)| {
                            let map =
                                StringMap::from_storage(storage, map.data.storage_index()).unwrap();
                            assert_eq!(map.capacity(), capacity);
                            assert_eq!(&string_pairs(&map, storage), pairs);
                        },
                    );
                }
            }

            #[test]
            fn rebuild_of_string_map_failure_should_keep_map_readable() {
                for (capacity, keys, removed, new_capacity) in STRING_CASES {
                    assert_io_failures_keep(
                        |storage| string_map(storage, capacity, keys.clone(), removed.clone()),
                        |storage, (map, _)| map.rebuild(storage, new_capacity, U32_SLOTS),
                        |storage, (map, pairs)| {
                            assert_eq!(&string_pairs(map, storage), pairs);
                        },
                    );
                }
            }

            type Range = std::ops::Range<u64>;

            /// Capacity, inserted keys, keys of which every fourth is removed
            /// and the capacity to rebuild to.
            const STRING_CASES: [(u64, Range, Range, u64); 2] =
                [(64, 0..40, 0..40, 128), (128, 0..20, 0..0, 64)];

            fn key(i: u64) -> String {
                format!("key {i}")
            }

            fn string_map(
                storage: &mut TestStorage,
                capacity: u64,
                keys: Range,
                removed: Range,
            ) -> (StringMap, Vec<(String, String)>) {
                let mut map = StringMap::new(storage).unwrap();
                map.reserve(storage, capacity).unwrap();
                for i in keys {
                    map.insert(storage, &key(i), &key(i)).unwrap();
                }
                for i in removed.step_by(4) {
                    map.remove_key(storage, &key(i)).unwrap();
                }
                let pairs = string_pairs(&map, storage);
                (map, pairs)
            }

            fn string_pairs(map: &StringMap, storage: &TestStorage) -> Vec<(String, String)> {
                let mut pairs = map.iter(storage).collect::<Vec<_>>();
                pairs.sort();
                pairs
            }
        }

        mod remove_key_impl {
            use super::*;

            #[test]
            fn remove_key_impl_unique_should_stop_after_first_match() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 10), (1, 20)]);

                map.remove_key_impl(&mut storage, &1, true).unwrap();

                assert_eq!(map.values(&storage, &1), Ok(vec![20]));
                assert_eq!(map.len(), 1);
            }
        }

        mod reserve_slot {
            use super::*;

            #[test]
            fn reserve_slot_with_few_deleted_should_grow() {
                let mut storage = storage();
                let mut map = filled(&mut storage, 0..60);
                map.remove_key(&mut storage, &0).unwrap();

                map.insert(&mut storage, &100, &100).unwrap();

                assert_eq!(map.capacity(), 128);
                assert_eq!(map.data.deleted(), 0);
            }

            #[test]
            fn reserve_slot_with_many_deleted_should_purge_in_place() {
                let mut storage = storage();
                let mut map = filled(&mut storage, 0..60);
                for i in 0..5 {
                    map.remove_key(&mut storage, &i).unwrap();
                }

                map.insert(&mut storage, &100, &100).unwrap();

                assert_eq!(map.capacity(), 64);
                assert_eq!(map.data.deleted(), 0);
                assert_eq!(map.len(), 56);
                for i in (5..60).chain([100]) {
                    assert_eq!(map.value(&storage, &i), Ok(Some(i)));
                }
            }
        }
    }

    mod multi_map_storage {
        use super::*;

        mod new {
            use super::*;

            #[test]
            fn new_should_create_empty_map() {
                let mut storage = storage();

                let map = TestMap::new(&mut storage).unwrap();

                assert_eq!(map.len(), 0);
                assert_eq!(map.capacity(), 0);
            }

            #[test]
            fn new_failure_should_fail() {
                assert_io_failures(|_| (), |storage, _| TestMap::new(storage));
            }
        }

        mod from_storage {
            use super::*;

            #[test]
            fn from_storage_should_load_values() {
                let mut storage = storage();
                let map = map(&mut storage, &[(1, 10), (1, 20), (2, 30)]);

                let map = TestMap::from_storage(&storage, map.storage_index()).unwrap();

                assert_eq!(sorted(&map, &storage), vec![(1, 10), (1, 20), (2, 30)]);
            }

            #[test]
            fn from_storage_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[(1, 10)]).storage_index(),
                    |storage, index| TestMap::from_storage(storage, *index),
                );
            }
        }
    }
}
