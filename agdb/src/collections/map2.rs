use super::multi_map2::DbMultiMap;
use super::multi_map2::DbMultiMapImpl;
use super::multi_map2::DbMultiMapImplMut;
use super::vec2::VecValue;
use crate::storage::file_storage::FileStorage;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::stable_hash::StableHash;
use crate::DbError;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum MapValueState {
    #[default]
    Empty,
    Deleted,
    Valid,
}

pub struct MapDataIndex {
    pub len: u64,
    pub states_index: StorageIndex,
    pub keys_index: StorageIndex,
    pub values_index: StorageIndex,
}

pub struct DbMap<K, T, S = FileStorage>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    multi_map: DbMultiMap<K, T, S>,
}

pub struct DbMapImpl<'a, K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    multi_map: DbMultiMapImpl<'a, K, T, S>,
}

pub struct DbMapImplMut<'a, K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    multi_map: DbMultiMapImplMut<'a, K, T, S>,
}

pub struct MapIterator<'a, K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    pub pos: u64,
    pub multi_map: &'a DbMultiMap<K, T, S>,
    pub storage: &'a S,
}

impl Serialize for MapValueState {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        match bytes.first() {
            Some(0) => Ok(MapValueState::Empty),
            Some(1) => Ok(MapValueState::Valid),
            Some(2) => Ok(MapValueState::Deleted),
            _ => Err(DbError::from(
                "MapValueState deserialization error: unknown value",
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

impl VecValue for MapValueState {
    fn store<S: Storage>(&self, _storage: &mut S) -> Result<Vec<u8>, DbError> {
        Ok(self.serialize())
    }

    fn load<S: Storage>(_storage: &S, bytes: &[u8]) -> Result<Self, DbError> {
        Self::deserialize(bytes)
    }

    fn remove<S: Storage>(_storage: &mut S, _bytes: &[u8]) -> Result<(), DbError> {
        Ok(())
    }

    fn storage_len() -> u64 {
        Self::serialized_size_static()
    }
}

impl SerializeStatic for MapDataIndex {
    fn serialized_size_static() -> u64 {
        u64::serialized_size_static() + StorageIndex::serialized_size_static() * 3
    }
}

impl Serialize for MapDataIndex {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(self.serialized_size() as usize);
        bytes.extend(self.len.serialize());
        bytes.extend(self.states_index.serialize());
        bytes.extend(self.keys_index.serialize());
        bytes.extend(self.values_index.serialize());

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        if bytes.len() < Self::serialized_size_static() as usize {
            return Err(DbError::from(
                "MapDataStorageIndex deserialization error: not enough data",
            ));
        }

        Ok(MapDataIndex {
            len: u64::deserialize(bytes)?,
            states_index: StorageIndex::deserialize(
                &bytes[u64::serialized_size_static() as usize..],
            )?,
            keys_index: StorageIndex::deserialize(
                &bytes[(u64::serialized_size_static() + StorageIndex::serialized_size_static())
                    as usize..],
            )?,
            values_index: StorageIndex::deserialize(
                &bytes[(u64::serialized_size_static() + StorageIndex::serialized_size_static() * 2)
                    as usize..],
            )?,
        })
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

impl<K, T, S> DbMap<K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.multi_map.capacity()
    }

    pub fn from_storage(storage: &S, storage_index: StorageIndex) -> Result<Self, DbError> {
        Ok(Self {
            multi_map: DbMultiMap::from_storage(storage, storage_index)?,
        })
    }

    pub fn len(&self) -> u64 {
        self.multi_map.len()
    }

    pub fn new(storage: &mut S) -> Result<Self, DbError> {
        Ok(Self {
            multi_map: DbMultiMap::new(storage)?,
        })
    }

    pub fn read<'a>(&'a self, storage: &'a S) -> DbMapImpl<'a, K, T, S> {
        DbMapImpl {
            multi_map: self.multi_map.read(storage),
        }
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.multi_map.storage_index()
    }

    pub fn write<'a>(&'a mut self, storage: &'a mut S) -> DbMapImplMut<'a, K, T, S> {
        DbMapImplMut {
            multi_map: self.multi_map.write(storage),
        }
    }
}

impl<'a, K, T, S> DbMapImpl<'a, K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.multi_map.capacity()
    }

    pub fn contains(&self, key: &K) -> Result<bool, DbError> {
        self.multi_map.contains(key)
    }

    pub fn contains_value(&self, key: &K, value: &T) -> Result<bool, DbError> {
        self.multi_map.contains_value(key, value)
    }

    pub fn is_empty(&self) -> bool {
        self.multi_map.len() == 0
    }

    pub fn iter(&self) -> MapIterator<'a, K, T, S> {
        self.multi_map.iter()
    }

    pub fn len(&self) -> u64 {
        self.multi_map.len()
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        self.multi_map.value(key)
    }
}

impl<'a, K, T, S> DbMapImplMut<'a, K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    pub fn capacity(&self) -> u64 {
        self.multi_map.capacity()
    }

    pub fn contains(&self, key: &K) -> Result<bool, DbError> {
        self.multi_map.contains(key)
    }

    pub fn contains_value(&self, key: &K, value: &T) -> Result<bool, DbError> {
        self.multi_map.contains_value(key, value)
    }

    pub fn insert(&mut self, key: &K, value: &T) -> Result<Option<T>, DbError> {
        self.multi_map.insert_or_replace(key, |_| true, value)
    }

    pub fn is_empty(&self) -> bool {
        self.multi_map.len() == 0
    }

    pub fn iter(&'a self) -> MapIterator<'a, K, T, S> {
        self.multi_map.iter()
    }

    pub fn len(&self) -> u64 {
        self.multi_map.len()
    }

    pub fn remove(&mut self, key: &K) -> Result<(), DbError> {
        self.multi_map.remove_key(key)
    }

    pub fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        self.multi_map.reserve(capacity)
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        self.multi_map.value(key)
    }
}

impl<'a, K, T, S> Iterator for MapIterator<'a, K, T, S>
where
    K: VecValue + Default + StableHash + PartialEq,
    T: VecValue + Default + PartialEq,
    S: Storage,
{
    type Item = (K, T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos != self.multi_map.capacity() {
            let current_pos = self.pos;
            self.pos += 1;

            if self
                .multi_map
                .state(self.storage, current_pos)
                .unwrap_or_default()
                == MapValueState::Valid
            {
                let key = self
                    .multi_map
                    .key_at(self.storage, current_pos)
                    .unwrap_or_default();
                let value = self
                    .multi_map
                    .value_at(self.storage, current_pos)
                    .unwrap_or_default();

                return Some((key, value));
            }
        }

        None
    }
}
