use crate::DbError;
use crate::DbErrorType;
use crate::StorageData;
use crate::collections::multi_map::MultiMapImpl;
use crate::collections::vec::DbVec;
use crate::collections::vec::VecValue;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::serialize::u64_at;
use crate::utilities::stable_hash::StableHash;
use std::marker::PhantomData;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum MapValueState {
    #[default]
    Empty,
    Deleted,
    Valid,
}

impl Serialize for MapValueState {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        match bytes.first() {
            Some(0) => Ok(MapValueState::Empty),
            Some(1) => Ok(MapValueState::Valid),
            Some(2) => Ok(MapValueState::Deleted),
            x => Err(DbError::collections(
                DbErrorType::TypeError,
                format!("MapValueState deserialization error. Unknown value: {x:?}"),
            )),
        }
    }

    fn serialize(&self) -> Vec<u8> {
        match self {
            MapValueState::Empty => vec![0_u8],
            MapValueState::Deleted => vec![2_u8],
            MapValueState::Valid => vec![1_u8],
        }
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

impl SerializeStatic for MapValueState {
    fn serialized_size_static() -> u64 {
        1
    }
}

impl<D: StorageData> VecValue<D> for MapValueState {
    fn store(&self, _storage: &mut Storage<D>) -> Result<Vec<u8>, DbError> {
        Ok(self.serialize())
    }

    fn load(_storage: &Storage<D>, bytes: &[u8]) -> Result<Self, DbError> {
        Self::deserialize(bytes)
    }

    fn remove(_storage: &mut Storage<D>, _bytes: &[u8]) -> Result<(), DbError> {
        Ok(())
    }

    fn storage_len() -> u64 {
        Self::serialized_size_static()
    }
}

pub trait MapData<K, T, D>
where
    D: StorageData,
{
    fn capacity(&self) -> u64;
    fn commit(&mut self, storage: &mut Storage<D>, id: u64) -> Result<(), DbError>;
    fn deleted(&self) -> u64;
    fn for_each_valid_key(
        &self,
        storage: &Storage<D>,
        f: impl FnMut(u64, K) -> Result<(), DbError>,
    ) -> Result<(), DbError>;
    fn len(&self) -> u64;
    fn rebuild<S: OldSlot>(
        &mut self,
        storage: &mut Storage<D>,
        old_slots: &[S],
    ) -> Result<(), DbError>;
    fn key(&self, storage: &Storage<D>, index: u64) -> Result<K, DbError>;
    fn remove_from_storage(self, storage: &mut Storage<D>) -> Result<(), DbError>;
    fn set_state(
        &mut self,
        storage: &mut Storage<D>,
        index: u64,
        state: MapValueState,
    ) -> Result<(), DbError>;
    fn set_key(&mut self, storage: &mut Storage<D>, index: u64, key: &K) -> Result<(), DbError>;
    fn set_value(&mut self, storage: &mut Storage<D>, index: u64, value: &T)
    -> Result<(), DbError>;
    fn shrink_to_fit(&mut self, storage: &mut Storage<D>) -> Result<(), DbError>;
    fn state(&self, storage: &Storage<D>, index: u64) -> Result<MapValueState, DbError>;
    fn transaction(&mut self, storage: &mut Storage<D>) -> u64;
    fn value(&self, storage: &Storage<D>, index: u64) -> Result<T, DbError>;
}

pub trait OldSlot: Copy + Eq {
    fn none() -> Self;
    fn new(slot: u64) -> Self;
    fn get(self) -> u64;
}

impl OldSlot for u32 {
    fn none() -> Self {
        u32::MAX
    }

    fn new(slot: u64) -> Self {
        slot as u32
    }

    fn get(self) -> u64 {
        u64::from(self)
    }
}

impl OldSlot for u64 {
    fn none() -> Self {
        u64::MAX
    }

    fn new(slot: u64) -> Self {
        slot
    }

    fn get(self) -> u64 {
        self
    }
}

#[cfg(test)]
const CHUNK: u64 = 64;
#[cfg(not(test))]
const CHUNK: u64 = 1024 * 1024;

struct SlotWindows<'a, V: VecValue<D>, D: StorageData> {
    vec: &'a DbVec<V, D>,
    windows: [(u64, Vec<u8>); 2],
    last: usize,
}

impl<'a, V: Clone + VecValue<D>, D: StorageData> SlotWindows<'a, V, D> {
    fn new(vec: &'a DbVec<V, D>) -> Self {
        Self {
            vec,
            windows: [(0, vec![]), (0, vec![])],
            last: 0,
        }
    }

    fn slot(&mut self, storage: &Storage<D>, slot: u64) -> Result<&[u8], DbError> {
        let len = V::storage_len();

        for (i, (start, bytes)) in self.windows.iter().enumerate() {
            if *start <= slot && slot < *start + bytes.len() as u64 / len {
                self.last = i;
                let offset = ((slot - start) * len) as usize;
                return Ok(&self.windows[i].1[offset..offset + len as usize]);
            }
        }

        let replaced = 1 - self.last;
        let count = std::cmp::min(std::cmp::max(1, CHUNK / len), self.vec.len() - slot);
        self.windows[replaced] = (slot, self.vec.bytes_at(storage, slot, count)?.to_vec());
        self.last = replaced;
        Ok(&self.windows[replaced].1[..len as usize])
    }
}

struct InvalidSlots<'a, D: StorageData> {
    states: &'a DbVec<MapValueState, D>,
    chunk: Vec<u8>,
    from: u64,
    next: u64,
}

impl<'a, D: StorageData> InvalidSlots<'a, D> {
    fn new(states: &'a DbVec<MapValueState, D>, from: u64) -> Self {
        Self {
            states,
            chunk: vec![],
            from,
            next: from,
        }
    }

    fn next(&mut self, storage: &Storage<D>) -> Result<Option<u64>, DbError> {
        let valid = MapValueState::Valid.serialize()[0];

        while self.next < self.states.len() {
            if self.next == self.from + self.chunk.len() as u64 {
                let count = std::cmp::min(CHUNK, self.states.len() - self.next);
                self.chunk = self.states.bytes_at(storage, self.next, count)?.to_vec();
                self.from = self.next;
            }

            let slot = self.next;
            self.next += 1;

            if self.chunk[(slot - self.from) as usize] != valid {
                return Ok(Some(slot));
            }
        }

        Ok(None)
    }
}

pub struct MapDataIndex {
    pub len: u64,
    pub states_index: StorageIndex,
    pub keys_index: StorageIndex,
    pub values_index: StorageIndex,
}

impl SerializeStatic for MapDataIndex {
    fn serialized_size_static() -> u64 {
        u64::serialized_size_static() + StorageIndex::serialized_size_static() * 3
    }
}

impl Serialize for MapDataIndex {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.serialized_size() as usize);
        bytes.extend(self.len.serialize());
        bytes.extend(self.states_index.serialize());
        bytes.extend(self.keys_index.serialize());
        bytes.extend(self.values_index.serialize());

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        if bytes.len() < Self::serialized_size_static() as usize {
            return Err(DbError::collections(
                DbErrorType::NotEnoughData,
                format!(
                    "MapDataStorageIndex deserialization error: got {} bytes, expected at least {} bytes",
                    bytes.len(),
                    Self::serialized_size_static(),
                ),
            ));
        }

        let field = |i: usize| u64_at(bytes, i);

        Ok(MapDataIndex {
            len: field(0),
            states_index: StorageIndex(field(1)),
            keys_index: StorageIndex(field(2)),
            values_index: StorageIndex(field(3)),
        })
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

pub struct DbMapData<K, T, D>
where
    K: Clone + VecValue<D>,
    T: Clone + VecValue<D>,
    D: StorageData,
{
    storage_index: StorageIndex,
    data_index: MapDataIndex,
    deleted: u64,
    states: DbVec<MapValueState, D>,
    keys: DbVec<K, D>,
    values: DbVec<T, D>,
}

impl<K, T, D> DbMapData<K, T, D>
where
    K: Clone + VecValue<D>,
    T: Clone + VecValue<D>,
    D: StorageData,
{
    pub fn new(storage: &mut Storage<D>) -> Result<Self, DbError> {
        let states = DbVec::<MapValueState, D>::new(storage)?;
        let keys = DbVec::<K, D>::new(storage)?;
        let values = DbVec::<T, D>::new(storage)?;

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
            deleted: 0,
            states,
            keys,
            values,
        })
    }

    pub fn from_storage(
        storage: &Storage<D>,
        storage_index: StorageIndex,
    ) -> Result<Self, DbError> {
        let mut data_index = storage.value::<MapDataIndex>(storage_index)?;
        let states = DbVec::<MapValueState, D>::from_storage(storage, data_index.states_index)?;
        let keys = DbVec::<K, D>::from_storage(storage, data_index.keys_index)?;
        let values = DbVec::<T, D>::from_storage(storage, data_index.values_index)?;
        let valid_state = MapValueState::Valid.serialize()[0];
        let deleted_state = MapValueState::Deleted.serialize()[0];
        let mut deleted = 0;
        data_index.len = 0;

        for from in (0..states.len()).step_by(CHUNK as usize) {
            let count = std::cmp::min(CHUNK, states.len() - from);

            for state in states.bytes_at(storage, from, count)?.iter() {
                if *state == valid_state {
                    data_index.len += 1;
                } else if *state == deleted_state {
                    deleted += 1;
                }
            }
        }

        Ok(Self {
            storage_index,
            data_index,
            deleted,
            states,
            keys,
            values,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_index
    }

    fn append_vec<S: OldSlot>(
        storage: &mut Storage<D>,
        old_slots: &[S],
        value_len: u64,
        mut fill: impl FnMut(&mut Storage<D>, &[S], &mut Vec<u8>) -> Result<(), DbError>,
    ) -> Result<StorageIndex, DbError> {
        let len = old_slots.len() as u64;
        let index = storage.insert_zeroed(u64::serialized_size_static() + len * value_len)?;
        storage.insert_at(index, 0, &len)?;
        let per_chunk = std::cmp::max(1, CHUNK / value_len) as usize;
        let mut bytes = Vec::with_capacity(per_chunk * value_len as usize);
        let mut offset = u64::serialized_size_static();

        for chunk in old_slots.chunks(per_chunk) {
            bytes.clear();
            fill(storage, chunk, &mut bytes)?;
            storage.insert_bytes_at(index, offset, &bytes)?;
            offset += bytes.len() as u64;
        }

        Ok(index)
    }

    fn append_slots<'a, V: Clone + Default + VecValue<D>, S: OldSlot>(
        storage: &mut Storage<D>,
        old: &DbVec<V, D>,
        states: &'a DbVec<MapValueState, D>,
        old_slots: &[S],
    ) -> Result<(StorageIndex, InvalidSlots<'a, D>), DbError> {
        let mut moved = SlotWindows::new(old);
        let mut taken = SlotWindows::new(old);
        let mut invalid = InvalidSlots::new(states, 0);
        let index = Self::append_vec(
            storage,
            old_slots,
            V::storage_len(),
            |storage, chunk, bytes| {
                for old_slot in chunk {
                    if *old_slot != S::none() {
                        bytes.extend_from_slice(moved.slot(storage, old_slot.get())?);
                    } else if let Some(slot) = invalid.next(storage)? {
                        bytes.extend_from_slice(taken.slot(storage, slot)?);
                    } else {
                        bytes.extend(V::default().store(storage)?);
                    }
                }

                Ok(())
            },
        )?;

        Ok((index, invalid))
    }
}

impl<K, T, D> MapData<K, T, D> for DbMapData<K, T, D>
where
    K: Default + Clone + VecValue<D>,
    T: Default + Clone + VecValue<D>,
    D: StorageData,
{
    fn capacity(&self) -> u64 {
        self.states.len()
    }

    fn commit(&mut self, storage: &mut Storage<D>, id: u64) -> Result<(), DbError> {
        storage.commit(id)
    }

    fn deleted(&self) -> u64 {
        self.deleted
    }

    fn for_each_valid_key(
        &self,
        storage: &Storage<D>,
        mut f: impl FnMut(u64, K) -> Result<(), DbError>,
    ) -> Result<(), DbError> {
        let valid = MapValueState::Valid.serialize()[0];
        let key_len = K::storage_len();
        let per_chunk = std::cmp::max(1, CHUNK / key_len);

        for from in (0..self.capacity()).step_by(per_chunk as usize) {
            let count = std::cmp::min(per_chunk, self.capacity() - from);
            let states = self.states.bytes_at(storage, from, count)?.to_vec();
            let keys = self.keys.bytes_at(storage, from, count)?;

            for (offset, key) in keys.chunks(key_len as usize).enumerate() {
                if states[offset] == valid {
                    f(from + offset as u64, K::load(storage, key)?)?;
                }
            }
        }

        Ok(())
    }

    fn len(&self) -> u64 {
        self.data_index.len
    }

    fn rebuild<S: OldSlot>(
        &mut self,
        storage: &mut Storage<D>,
        old_slots: &[S],
    ) -> Result<(), DbError> {
        let id = storage.transaction();
        let valid = MapValueState::Valid.serialize()[0];
        let empty = MapValueState::Empty.serialize()[0];
        let states = Self::append_vec(storage, old_slots, 1, |_, chunk, bytes| {
            bytes.extend(chunk.iter().map(
                |old_slot| {
                    if *old_slot == S::none() { empty } else { valid }
                },
            ));
            Ok(())
        })?;
        let (keys, _) = Self::append_slots(storage, &self.keys, &self.states, old_slots)?;
        let (values, remaining) =
            Self::append_slots(storage, &self.values, &self.states, old_slots)?;
        let remaining = remaining.next;
        let data_index = MapDataIndex {
            len: self.data_index.len,
            states_index: states,
            keys_index: keys,
            values_index: values,
        };
        let states = DbVec::from_storage(storage, states)?;
        let keys = DbVec::from_storage(storage, keys)?;
        let values = DbVec::from_storage(storage, values)?;
        storage.insert_at(self.storage_index, 0, &data_index)?;

        self.data_index = data_index;
        self.deleted = 0;
        let old_states = std::mem::replace(&mut self.states, states);
        let old_keys = std::mem::replace(&mut self.keys, keys);
        let old_values = std::mem::replace(&mut self.values, values);

        let mut remaining = InvalidSlots::new(&old_states, remaining);
        let mut old_key_slots = SlotWindows::new(&old_keys);
        let mut old_value_slots = SlotWindows::new(&old_values);

        while let Some(slot) = remaining.next(storage)? {
            let key = old_key_slots.slot(storage, slot)?.to_vec();
            K::remove(storage, &key)?;
            let value = old_value_slots.slot(storage, slot)?.to_vec();
            T::remove(storage, &value)?;
        }

        storage.remove(old_states.storage_index())?;
        storage.remove(old_keys.storage_index())?;
        storage.remove(old_values.storage_index())?;
        storage.commit(id)
    }

    fn key(&self, storage: &Storage<D>, index: u64) -> Result<K, DbError> {
        self.keys.value(storage, index)
    }

    fn remove_from_storage(self, storage: &mut Storage<D>) -> Result<(), DbError> {
        let id = storage.transaction();
        self.states.remove_from_storage(storage)?;
        self.values.remove_from_storage(storage)?;
        self.keys.remove_from_storage(storage)?;
        storage.remove(self.storage_index)?;
        storage.commit(id)
    }

    fn set_state(
        &mut self,
        storage: &mut Storage<D>,
        index: u64,
        state: MapValueState,
    ) -> Result<(), DbError> {
        let old_state = self.states.replace(storage, index, &state)?;

        match old_state {
            MapValueState::Empty => {}
            MapValueState::Deleted => self.deleted -= 1,
            MapValueState::Valid => self.data_index.len -= 1,
        }

        match state {
            MapValueState::Empty => {}
            MapValueState::Deleted => self.deleted += 1,
            MapValueState::Valid => self.data_index.len += 1,
        }

        Ok(())
    }

    fn set_key(&mut self, storage: &mut Storage<D>, index: u64, key: &K) -> Result<(), DbError> {
        self.keys.replace(storage, index, key)?;
        Ok(())
    }

    fn set_value(
        &mut self,
        storage: &mut Storage<D>,
        index: u64,
        value: &T,
    ) -> Result<(), DbError> {
        self.values.replace(storage, index, value)?;
        Ok(())
    }

    fn shrink_to_fit(&mut self, storage: &mut Storage<D>) -> Result<(), DbError> {
        self.states.shrink_to_fit(storage)?;
        self.keys.shrink_to_fit(storage)?;
        self.values.shrink_to_fit(storage)
    }

    fn state(&self, storage: &Storage<D>, index: u64) -> Result<MapValueState, DbError> {
        self.states.value(storage, index)
    }

    fn transaction(&mut self, storage: &mut Storage<D>) -> u64 {
        storage.transaction()
    }

    fn value(&self, storage: &Storage<D>, index: u64) -> Result<T, DbError> {
        self.values.value(storage, index)
    }
}

pub struct MapIterator<'a, K, T, D, Data>
where
    D: StorageData,
    Data: MapData<K, T, D>,
{
    pub pos: u64,
    pub data: &'a Data,
    pub storage: &'a Storage<D>,
    pub phantom_data: PhantomData<(K, T, D)>,
}

impl<K, T, D, Data> Iterator for MapIterator<'_, K, T, D, Data>
where
    K: Default,
    T: Default,
    D: StorageData,
    Data: MapData<K, T, D>,
{
    type Item = (K, T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos != self.data.capacity() {
            let current_pos = self.pos;
            self.pos += 1;

            if self
                .data
                .state(self.storage, current_pos)
                .unwrap_or_default()
                == MapValueState::Valid
            {
                let key = self.data.key(self.storage, current_pos).unwrap_or_default();
                let value = self
                    .data
                    .value(self.storage, current_pos)
                    .unwrap_or_default();

                return Some((key, value));
            }
        }

        None
    }
}

pub struct MapImpl<K, T, D, Data>
where
    D: StorageData,
    Data: MapData<K, T, D>,
{
    multi_map: MultiMapImpl<K, T, D, Data>,
    storage: PhantomData<D>,
}

impl<K, T, D, Data> MapImpl<K, T, D, Data>
where
    K: Default + PartialEq + StableHash,
    T: Default + PartialEq,
    D: StorageData,
    Data: MapData<K, T, D>,
{
    #[allow(dead_code)]
    pub fn capacity(&self) -> u64 {
        self.multi_map.capacity()
    }

    #[allow(dead_code)]
    pub fn contains(&self, storage: &Storage<D>, key: &K) -> Result<bool, DbError> {
        self.multi_map.contains(storage, key)
    }

    #[allow(dead_code)]
    pub fn contains_value(
        &self,
        storage: &Storage<D>,
        key: &K,
        value: &T,
    ) -> Result<bool, DbError> {
        Ok(self.multi_map.value(storage, key)?.as_ref() == Some(value))
    }

    pub fn insert(
        &mut self,
        storage: &mut Storage<D>,
        key: &K,
        value: &T,
    ) -> Result<Option<T>, DbError> {
        self.multi_map
            .insert_or_replace(storage, key, |_| true, value)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.multi_map.is_empty()
    }

    pub fn iter<'a>(&'a self, storage: &'a Storage<D>) -> MapIterator<'a, K, T, D, Data> {
        self.multi_map.iter(storage)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> u64 {
        self.multi_map.len()
    }

    pub fn remove(&mut self, storage: &mut Storage<D>, key: &K) -> Result<(), DbError> {
        self.multi_map.remove_key_impl(storage, key, true)
    }

    #[allow(dead_code)]
    pub fn reserve(&mut self, storage: &mut Storage<D>, capacity: u64) -> Result<(), DbError> {
        self.multi_map.reserve(storage, capacity)
    }

    pub fn shrink_to_fit(&mut self, storage: &mut Storage<D>) -> Result<(), DbError> {
        self.multi_map.shrink_to_fit(storage)
    }

    pub fn value(&self, storage: &Storage<D>, key: &K) -> Result<Option<T>, DbError> {
        self.multi_map.value(storage, key)
    }
}

pub type DbMap<K, T, D> = MapImpl<K, T, D, DbMapData<K, T, D>>;

impl<K, T, D> DbMap<K, T, D>
where
    K: Default + Clone + VecValue<D>,
    T: Default + Clone + VecValue<D>,
    D: StorageData,
{
    pub fn new(storage: &mut Storage<D>) -> Result<Self, DbError> {
        Ok(Self {
            multi_map: MultiMapImpl::<K, T, D, DbMapData<K, T, D>> {
                data: DbMapData::<K, T, D>::new(storage)?,
                phantom_marker: PhantomData,
            },
            storage: PhantomData,
        })
    }

    pub fn from_storage(storage: &Storage<D>, index: StorageIndex) -> Result<Self, DbError> {
        Ok(Self {
            multi_map: MultiMapImpl::<K, T, D, DbMapData<K, T, D>> {
                data: DbMapData::<K, T, D>::from_storage(storage, index)?,
                phantom_marker: PhantomData,
            },
            storage: PhantomData,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.multi_map.data.storage_index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::tests::TestStorage;
    use crate::collections::tests::add_spare_capacity;
    use crate::collections::tests::assert_io_failures;
    use crate::collections::tests::storage;
    use crate::storage::file_storage::FileStorageImpl;
    use agplatform::test_platform::TestPlatform;

    type D = FileStorageImpl<TestPlatform>;
    type TestMap = DbMap<u64, u64, D>;

    fn map(storage: &mut TestStorage, pairs: &[(u64, u64)]) -> TestMap {
        let mut map = TestMap::new(storage).unwrap();

        for (key, value) in pairs {
            map.insert(storage, key, value).unwrap();
        }

        map
    }

    fn filled(storage: &mut TestStorage, len: u64) -> TestMap {
        map(storage, &(0..len).map(|i| (i, i)).collect::<Vec<_>>())
    }

    fn sorted(map: &TestMap, storage: &TestStorage) -> Vec<(u64, u64)> {
        let mut pairs = map.iter(storage).collect::<Vec<_>>();
        pairs.sort();
        pairs
    }

    mod map_value_state {
        use super::*;

        mod serialize {
            use super::*;

            mod deserialize {
                use super::*;

                #[test]
                fn deserialize_should_read_each_state() {
                    assert_eq!(MapValueState::deserialize(&[0]), Ok(MapValueState::Empty));
                    assert_eq!(MapValueState::deserialize(&[1]), Ok(MapValueState::Valid));
                    assert_eq!(MapValueState::deserialize(&[2]), Ok(MapValueState::Deleted));
                }

                #[test]
                fn deserialize_unknown_value_should_fail() {
                    assert_eq!(
                        MapValueState::deserialize(&[3]),
                        Err(DbError::collections(
                            DbErrorType::TypeError,
                            "MapValueState deserialization error. Unknown value: Some(3)"
                        ))
                    );
                }

                #[test]
                fn deserialize_empty_bytes_should_fail() {
                    assert_eq!(
                        MapValueState::deserialize(&[]),
                        Err(DbError::collections(
                            DbErrorType::TypeError,
                            "MapValueState deserialization error. Unknown value: None"
                        ))
                    );
                }
            }

            #[allow(clippy::module_inception)]
            mod serialize {
                use super::*;

                #[test]
                fn serialize_should_write_one_byte_per_state() {
                    assert_eq!(MapValueState::Empty.serialize(), vec![0]);
                    assert_eq!(MapValueState::Valid.serialize(), vec![1]);
                    assert_eq!(MapValueState::Deleted.serialize(), vec![2]);
                }
            }

            mod serialized_size {
                use super::*;

                #[test]
                fn serialized_size_should_be_one() {
                    assert_eq!(MapValueState::Valid.serialized_size(), 1);
                }
            }
        }

        mod serialize_static {
            use super::*;

            mod serialized_size_static {
                use super::*;

                #[test]
                fn serialized_size_static_should_be_one() {
                    assert_eq!(MapValueState::serialized_size_static(), 1);
                }
            }
        }

        mod vec_value {
            use super::*;

            mod store {
                use super::*;

                #[test]
                fn store_should_serialize_state() {
                    let mut storage = storage();

                    assert_eq!(MapValueState::Deleted.store(&mut storage), Ok(vec![2]));
                }
            }

            mod load {
                use super::*;

                #[test]
                fn load_should_deserialize_state() {
                    let storage = storage();

                    assert_eq!(
                        <MapValueState as VecValue<D>>::load(&storage, &[1]),
                        Ok(MapValueState::Valid)
                    );
                }
            }

            mod remove {
                use super::*;

                #[test]
                fn remove_should_do_nothing() {
                    let mut storage = storage();
                    let len = storage.len();

                    <MapValueState as VecValue<D>>::remove(&mut storage, &[1]).unwrap();

                    assert_eq!(storage.len(), len);
                }
            }

            mod storage_len {
                use super::*;

                #[test]
                fn storage_len_should_be_one() {
                    assert_eq!(<MapValueState as VecValue<D>>::storage_len(), 1);
                }
            }
        }
    }

    mod map_data_index {
        use super::*;

        mod serialize_static {
            use super::*;

            mod serialized_size_static {
                use super::*;

                #[test]
                fn serialized_size_static_should_be_four_words() {
                    assert_eq!(MapDataIndex::serialized_size_static(), 32);
                }
            }
        }

        mod serialize {
            use super::*;

            fn index() -> MapDataIndex {
                MapDataIndex {
                    len: 1,
                    states_index: StorageIndex::from(2_u64),
                    keys_index: StorageIndex::from(3_u64),
                    values_index: StorageIndex::from(4_u64),
                }
            }

            #[allow(clippy::module_inception)]
            mod serialize {
                use super::*;

                #[test]
                fn serialize_should_write_fields_in_order() {
                    assert_eq!(
                        index().serialize(),
                        [1_u64, 2, 3, 4].map(|field| field.serialize()).concat()
                    );
                }
            }

            mod deserialize {
                use super::*;

                #[test]
                fn deserialize_should_read_serialized_fields() {
                    let index = MapDataIndex::deserialize(&index().serialize()).unwrap();

                    assert_eq!(index.len, 1);
                    assert_eq!(index.states_index, StorageIndex::from(2_u64));
                    assert_eq!(index.keys_index, StorageIndex::from(3_u64));
                    assert_eq!(index.values_index, StorageIndex::from(4_u64));
                }

                #[test]
                fn deserialize_short_bytes_should_fail() {
                    assert_eq!(
                        MapDataIndex::deserialize(&[0; 31]).err().unwrap(),
                        DbError::collections(
                            DbErrorType::NotEnoughData,
                            "MapDataStorageIndex deserialization error: got 31 bytes, expected at least 32 bytes"
                        )
                    );
                }
            }

            mod serialized_size {
                use super::*;

                #[test]
                fn serialized_size_should_be_four_words() {
                    assert_eq!(index().serialized_size(), 32);
                }
            }
        }
    }

    mod db_map_data {
        use super::*;

        type TestData = DbMapData<u64, u64, D>;

        mod new {
            use super::*;

            #[test]
            fn new_should_create_empty_data() {
                let mut storage = storage();

                let data = TestData::new(&mut storage).unwrap();

                assert_eq!(data.len(), 0);
                assert_eq!(data.capacity(), 0);
                assert_eq!(data.deleted(), 0);
            }

            #[test]
            fn new_failure_should_fail() {
                assert_io_failures(|_| (), |storage, _| TestData::new(storage));
            }
        }

        mod from_storage {
            use super::*;

            #[test]
            fn from_storage_should_count_valid_and_deleted_states_of_all_chunks() {
                let mut storage = storage();
                let mut map = map(&mut storage, &(0..100).map(|i| (i, i)).collect::<Vec<_>>());
                map.remove(&mut storage, &2).unwrap();
                map.remove(&mut storage, &99).unwrap();
                storage.insert_at(map.storage_index(), 0, &7_u64).unwrap();

                let data = TestData::from_storage(&storage, map.storage_index()).unwrap();

                assert_eq!(data.len(), 98);
                assert_eq!(data.capacity(), 128);
                assert_eq!(data.deleted(), 2);
            }

            #[test]
            fn from_storage_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[(1, 10)]).storage_index(),
                    |storage, index| TestData::from_storage(storage, *index),
                );
            }
        }

        mod map_data {
            use super::*;

            mod for_each_valid_key {
                use super::*;

                #[test]
                fn for_each_valid_key_should_visit_valid_slots_in_order() {
                    let mut storage = storage();
                    let mut map = map(&mut storage, &[(3, 30), (1, 10), (2, 20)]);
                    map.remove(&mut storage, &2).unwrap();
                    let data = TestData::from_storage(&storage, map.storage_index()).unwrap();
                    let mut keys = vec![];

                    data.for_each_valid_key(&storage, |slot, key| {
                        keys.push((slot, key));
                        Ok(())
                    })
                    .unwrap();

                    assert_eq!(keys, vec![(1, 1), (3, 3)]);
                }

                #[test]
                fn for_each_valid_key_failure_should_fail() {
                    assert_io_failures(
                        |storage| {
                            let map = map(storage, &[(1, 10)]);
                            TestData::from_storage(storage, map.storage_index()).unwrap()
                        },
                        |storage, data| data.for_each_valid_key(storage, |_, _| Ok(())),
                    );
                }
            }

            mod rebuild {
                use super::*;

                #[test]
                fn rebuild_should_move_valid_slots_and_empty_others() {
                    let mut storage = storage();
                    let mut map = map(&mut storage, &[(1, 10), (2, 20), (3, 30)]);
                    map.remove(&mut storage, &2).unwrap();
                    let mut data = TestData::from_storage(&storage, map.storage_index()).unwrap();

                    data.rebuild(&mut storage, &[u32::none(), 3, u32::none(), 1])
                        .unwrap();

                    let data = TestData::from_storage(&storage, data.storage_index()).unwrap();
                    assert_eq!(data.capacity(), 4);
                    assert_eq!(data.len(), 2);
                    assert_eq!(data.deleted(), 0);
                    assert_eq!(
                        (0..4)
                            .map(|i| data.state(&storage, i).unwrap())
                            .collect::<Vec<_>>(),
                        vec![
                            MapValueState::Empty,
                            MapValueState::Valid,
                            MapValueState::Empty,
                            MapValueState::Valid
                        ]
                    );
                    assert_eq!(
                        (data.key(&storage, 1), data.value(&storage, 1)),
                        (Ok(3), Ok(30))
                    );
                    assert_eq!(
                        (data.key(&storage, 3), data.value(&storage, 3)),
                        (Ok(1), Ok(10))
                    );
                }

                #[test]
                fn rebuild_failure_should_fail() {
                    assert_io_failures(
                        |storage| {
                            let map = map(storage, &[(1, 10), (2, 20)]);
                            TestData::from_storage(storage, map.storage_index()).unwrap()
                        },
                        |storage, data| data.rebuild(storage, &[2_u32, u32::none(), 1]),
                    );
                }
            }

            mod remove_from_storage {
                use super::*;

                #[test]
                fn remove_from_storage_should_remove_all_records() {
                    let mut storage = storage();
                    let len = storage.len();
                    let data = TestData::new(&mut storage).unwrap();

                    data.remove_from_storage(&mut storage).unwrap();

                    storage.optimize_storage().unwrap();
                    assert_eq!(storage.len(), len);
                }

                #[test]
                fn remove_from_storage_failure_should_fail() {
                    assert_io_failures(
                        |storage| Some(TestData::new(storage).unwrap()),
                        |storage, data| data.take().unwrap().remove_from_storage(storage),
                    );
                }
            }

            mod set_state {
                use super::*;

                #[test]
                fn set_state_should_count_valid_and_deleted_states() {
                    let mut storage = storage();
                    let mut data = TestData::new(&mut storage).unwrap();
                    data.rebuild(&mut storage, &[u32::none(); 2]).unwrap();

                    data.set_state(&mut storage, 0, MapValueState::Deleted)
                        .unwrap();
                    data.set_state(&mut storage, 1, MapValueState::Deleted)
                        .unwrap();
                    data.set_state(&mut storage, 1, MapValueState::Deleted)
                        .unwrap();
                    data.set_state(&mut storage, 0, MapValueState::Valid)
                        .unwrap();
                    data.set_state(&mut storage, 1, MapValueState::Valid)
                        .unwrap();
                    data.set_state(&mut storage, 1, MapValueState::Empty)
                        .unwrap();

                    assert_eq!(data.len(), 1);
                    assert_eq!(data.deleted(), 0);
                    assert_eq!(data.state(&storage, 0), Ok(MapValueState::Valid));
                    assert_eq!(data.state(&storage, 1), Ok(MapValueState::Empty));
                }

                #[test]
                fn set_state_failure_should_keep_counts() {
                    assert_io_failures(
                        |storage| {
                            let mut data = TestData::new(storage).unwrap();
                            data.rebuild(storage, &[u32::none(); 1]).unwrap();
                            data
                        },
                        |storage, data| {
                            let result = data.set_state(storage, 0, MapValueState::Valid);
                            assert_eq!(data.len(), u64::from(result.is_ok()));
                            result
                        },
                    );
                }
            }
        }
    }

    mod map_iterator {
        use super::*;

        mod iterator {
            use super::*;

            mod next {
                use super::*;

                #[test]
                fn next_should_skip_empty_and_deleted_slots() {
                    let mut storage = storage();
                    let mut map = map(&mut storage, &[(1, 10), (5, 15), (7, 20), (2, 30)]);
                    map.remove(&mut storage, &7).unwrap();

                    assert_eq!(sorted(&map, &storage), vec![(1, 10), (2, 30), (5, 15)]);
                }

                #[test]
                fn next_of_new_map_should_be_none() {
                    let mut storage = storage();
                    let map = map(&mut storage, &[]);

                    assert_eq!(map.iter(&storage).next(), None);
                }
            }
        }
    }

    mod map_impl {
        use super::*;

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
            fn contains_should_find_inserted_key() {
                let mut storage = storage();
                let map = map(&mut storage, &[(1, 10)]);

                assert_eq!(map.contains(&storage, &1), Ok(true));
            }

            #[test]
            fn contains_missing_key_should_be_false() {
                let mut storage = storage();

                assert_eq!(map(&mut storage, &[]).contains(&storage, &1), Ok(false));
                assert_eq!(
                    map(&mut storage, &[(1, 10)]).contains(&storage, &2),
                    Ok(false)
                );
            }

            #[test]
            fn contains_removed_key_should_be_false() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 10)]);
                map.remove(&mut storage, &1).unwrap();

                assert_eq!(map.contains(&storage, &1), Ok(false));
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
            fn contains_value_should_match_key_and_value() {
                let mut storage = storage();
                let map = map(&mut storage, &[(1, 10)]);

                assert_eq!(map.contains_value(&storage, &1, &10), Ok(true));
                assert_eq!(map.contains_value(&storage, &1, &11), Ok(false));
                assert_eq!(map.contains_value(&storage, &2, &10), Ok(false));
            }

            #[test]
            fn contains_value_of_removed_key_should_be_false() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 10)]);
                map.remove(&mut storage, &1).unwrap();

                assert_eq!(map.contains_value(&storage, &1, &10), Ok(false));
            }

            #[test]
            fn contains_value_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[(1, 10)]),
                    |storage, map| map.contains_value(storage, &1, &10),
                );
            }
        }

        mod insert {
            use super::*;

            #[test]
            fn insert_should_store_values() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[]);

                assert_eq!(map.insert(&mut storage, &1, &10), Ok(None));
                assert_eq!(map.insert(&mut storage, &5, &15), Ok(None));
                assert_eq!(map.insert(&mut storage, &7, &20), Ok(None));

                assert_eq!(map.len(), 3);
                assert_eq!(sorted(&map, &storage), vec![(1, 10), (5, 15), (7, 20)]);
            }

            #[test]
            fn insert_same_key_should_replace_value() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 10), (5, 15)]);

                assert_eq!(map.insert(&mut storage, &5, &20), Ok(Some(15)));

                assert_eq!(map.len(), 2);
                assert_eq!(sorted(&map, &storage), vec![(1, 10), (5, 20)]);
            }

            #[test]
            fn insert_over_max_len_should_double_capacity() {
                let mut storage = storage();

                let map = filled(&mut storage, 100);

                assert_eq!(map.len(), 100);
                assert_eq!(map.capacity(), 128);
                for i in 0..100 {
                    assert_eq!(map.value(&storage, &i), Ok(Some(i)));
                }
            }

            #[test]
            fn insert_colliding_keys_should_keep_all_values() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[]);

                for i in 1..100 {
                    map.insert(&mut storage, &(i * 64 - 1), &i).unwrap();
                }

                for i in 1..100 {
                    assert_eq!(map.value(&storage, &(i * 64 - 1)), Ok(Some(i)));
                }
            }

            #[test]
            fn insert_colliding_at_last_slot_should_wrap_around() {
                let mut storage = storage();

                let map = map(&mut storage, &[(127, 10), (255, 11), (191, 12)]);

                assert_eq!(map.value(&storage, &127), Ok(Some(10)));
                assert_eq!(map.value(&storage, &255), Ok(Some(11)));
                assert_eq!(map.value(&storage, &191), Ok(Some(12)));
            }

            #[test]
            fn insert_failure_should_fail() {
                assert_io_failures(
                    |storage| filled(storage, 60),
                    |storage, map| map.insert(storage, &60, &60),
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

                map.remove(&mut storage, &1).unwrap();
                assert!(map.is_empty());
            }
        }

        mod iter {
            use super::*;

            #[test]
            fn iter_should_yield_all_pairs() {
                let mut storage = storage();
                let map = map(&mut storage, &[(1, 10), (2, 20)]);

                assert_eq!(sorted(&map, &storage), vec![(1, 10), (2, 20)]);
            }
        }

        mod len {
            use super::*;

            #[test]
            fn len_should_count_keys() {
                let mut storage = storage();

                assert_eq!(map(&mut storage, &[(1, 10), (1, 11), (2, 20)]).len(), 2);
            }
        }

        mod remove {
            use super::*;

            #[test]
            fn remove_should_remove_only_key() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 10), (5, 15), (7, 20)]);

                map.remove(&mut storage, &5).unwrap();

                assert_eq!(map.len(), 2);
                assert_eq!(sorted(&map, &storage), vec![(1, 10), (7, 20)]);
            }

            #[test]
            fn remove_removed_key_should_do_nothing() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 10), (5, 15)]);
                map.remove(&mut storage, &5).unwrap();

                map.remove(&mut storage, &5).unwrap();

                assert_eq!(map.len(), 1);
            }

            #[test]
            fn remove_missing_key_should_do_nothing() {
                let mut storage = storage();
                let mut empty = map(&mut storage, &[]);
                let mut map = map(&mut storage, &[(1, 10)]);

                assert_eq!(empty.remove(&mut storage, &0), Ok(()));
                assert_eq!(map.remove(&mut storage, &0), Ok(()));

                assert_eq!(map.len(), 1);
            }

            #[test]
            fn remove_under_min_len_should_halve_capacity() {
                let mut storage = storage();
                let mut map = filled(&mut storage, 100);

                for i in (0..100).rev() {
                    map.remove(&mut storage, &i).unwrap();
                }

                assert_eq!(map.len(), 0);
                assert_eq!(map.capacity(), 64);
            }

            #[test]
            fn remove_failure_should_fail() {
                assert_io_failures(
                    |storage| filled(storage, 57),
                    |storage, map| map.remove(storage, &0),
                );
            }
        }

        mod reserve {
            use super::*;

            #[test]
            fn reserve_larger_should_double_capacity_to_fit() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 1)]);

                map.reserve(&mut storage, 74).unwrap();

                assert_eq!(map.capacity(), 128);
                assert_eq!(map.len(), 1);
                assert_eq!(map.value(&storage, &1), Ok(Some(1)));
            }

            #[test]
            fn reserve_same_or_smaller_should_keep_capacity() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 1)]);

                map.reserve(&mut storage, 64).unwrap();
                map.reserve(&mut storage, 54).unwrap();

                assert_eq!(map.capacity(), 64);
                assert_eq!(map.len(), 1);
            }

            #[test]
            fn reserve_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[(1, 1)]),
                    |storage, map| map.reserve(storage, 128),
                );
            }
        }

        mod shrink_to_fit {
            use super::*;

            fn spare(storage: &mut TestStorage) -> TestMap {
                let map = filled(storage, 10);
                add_spare_capacity(storage, map.storage_index());
                TestMap::from_storage(storage, map.storage_index()).unwrap()
            }

            #[test]
            fn shrink_to_fit_should_keep_capacity_and_values() {
                let mut storage = storage();
                let mut map = spare(&mut storage);
                let len = storage.len();

                map.shrink_to_fit(&mut storage).unwrap();

                storage.optimize_storage().unwrap();
                assert!(storage.len() < len);
                assert_eq!(map.capacity(), 64);
                assert_eq!(
                    sorted(&map, &storage),
                    (0..10).map(|i| (i, i)).collect::<Vec<_>>()
                );
            }

            #[test]
            fn shrink_to_fit_failure_should_fail() {
                assert_io_failures(spare, |storage, map| map.shrink_to_fit(storage));
            }
        }

        mod value {
            use super::*;

            #[test]
            fn value_should_return_inserted_value() {
                let mut storage = storage();
                let map = map(&mut storage, &[(1, 10)]);

                assert_eq!(map.value(&storage, &1), Ok(Some(10)));
            }

            #[test]
            fn value_missing_should_be_none() {
                let mut storage = storage();

                assert_eq!(map(&mut storage, &[]).value(&storage, &0), Ok(None));
                assert_eq!(map(&mut storage, &[(1, 10)]).value(&storage, &0), Ok(None));
            }

            #[test]
            fn value_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[(1, 10)]),
                    |storage, map| map.value(storage, &1),
                );
            }
        }
    }

    mod db_map {
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
            fn from_storage_should_load_pairs() {
                let mut storage = storage();
                let mut map = map(&mut storage, &[(1, 1), (3, 2), (5, 3)]);
                map.remove(&mut storage, &3).unwrap();

                let map = TestMap::from_storage(&storage, map.storage_index()).unwrap();

                assert_eq!(sorted(&map, &storage), vec![(1, 1), (5, 3)]);
            }

            #[test]
            fn from_storage_missing_index_should_fail() {
                let storage = storage();

                assert_eq!(
                    TestMap::from_storage(&storage, StorageIndex::from(1_u64))
                        .err()
                        .unwrap(),
                    DbError::collections(DbErrorType::NotFound, "Index (1) not found")
                );
            }

            #[test]
            fn from_storage_failure_should_fail() {
                assert_io_failures(
                    |storage| map(storage, &[(1, 10)]).storage_index(),
                    |storage, index| TestMap::from_storage(storage, *index),
                );
            }
        }

        mod storage_index {
            use super::*;

            #[test]
            fn storage_index_should_differ_per_map() {
                let mut storage = storage();

                let first = map(&mut storage, &[]).storage_index();
                let second = map(&mut storage, &[]).storage_index();

                assert_ne!(first, second);
            }
        }
    }
}
