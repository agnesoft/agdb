use crate::collections::multi_map::MultiMapImpl;
use crate::collections::vec::DbVec;
use crate::collections::vec::VecValue;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::stable_hash::StableHash;
use crate::DbError;
use crate::StorageData;
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
    fn store<D: StorageData>(&self, _storage: &mut Storage<D>) -> Result<Vec<u8>, DbError> {
        Ok(self.serialize())
    }

    fn load<D: StorageData>(_storage: &Storage<D>, bytes: &[u8]) -> Result<Self, DbError> {
        Self::deserialize(bytes)
    }

    fn remove<D: StorageData>(_storage: &mut Storage<D>, _bytes: &[u8]) -> Result<(), DbError> {
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
    fn len(&self) -> u64;
    fn key(&self, storage: &Storage<D>, index: u64) -> Result<K, DbError>;
    fn resize(&mut self, storage: &mut Storage<D>, capacity: u64) -> Result<(), DbError>;
    fn set_len(&mut self, storage: &mut Storage<D>, len: u64) -> Result<(), DbError>;
    fn set_state(
        &mut self,
        storage: &mut Storage<D>,
        index: u64,
        state: MapValueState,
    ) -> Result<(), DbError>;
    fn set_key(&mut self, storage: &mut Storage<D>, index: u64, key: &K) -> Result<(), DbError>;
    fn set_value(&mut self, storage: &mut Storage<D>, index: u64, value: &T)
        -> Result<(), DbError>;
    fn state(&self, storage: &Storage<D>, index: u64) -> Result<MapValueState, DbError>;
    fn swap(&mut self, storage: &mut Storage<D>, index: u64, other: u64) -> Result<(), DbError>;
    fn transaction(&mut self, storage: &mut Storage<D>) -> u64;
    fn value(&self, storage: &Storage<D>, index: u64) -> Result<T, DbError>;
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

pub struct DbMapData<K, T, D>
where
    K: Clone + VecValue,
    T: Clone + VecValue,
    D: StorageData,
{
    storage_index: StorageIndex,
    data_index: MapDataIndex,
    states: DbVec<MapValueState, D>,
    keys: DbVec<K, D>,
    values: DbVec<T, D>,
}

impl<K, T, D> DbMapData<K, T, D>
where
    K: Clone + VecValue,
    T: Clone + VecValue,
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
            states,
            keys,
            values,
        })
    }

    pub fn from_storage(
        storage: &Storage<D>,
        storage_index: StorageIndex,
    ) -> Result<Self, DbError> {
        let data_index = storage.value::<MapDataIndex>(storage_index)?;
        let states = DbVec::<MapValueState, D>::from_storage(storage, data_index.states_index)?;
        let keys = DbVec::<K, D>::from_storage(storage, data_index.keys_index)?;
        let values = DbVec::<T, D>::from_storage(storage, data_index.values_index)?;

        Ok(Self {
            storage_index,
            data_index,
            states,
            keys,
            values,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_index
    }
}

impl<K, T, D> MapData<K, T, D> for DbMapData<K, T, D>
where
    K: Default + Clone + VecValue,
    T: Default + Clone + VecValue,
    D: StorageData,
{
    fn capacity(&self) -> u64 {
        self.states.len()
    }

    fn commit(&mut self, storage: &mut Storage<D>, id: u64) -> Result<(), DbError> {
        storage.commit(id)
    }

    fn len(&self) -> u64 {
        self.data_index.len
    }

    fn key(&self, storage: &Storage<D>, index: u64) -> Result<K, DbError> {
        self.keys.value(storage, index)
    }

    fn resize(&mut self, storage: &mut Storage<D>, capacity: u64) -> Result<(), DbError> {
        self.states
            .resize(storage, capacity, &MapValueState::Empty)?;
        self.keys.resize(storage, capacity, &K::default())?;
        self.values.resize(storage, capacity, &T::default())
    }

    fn set_len(&mut self, storage: &mut Storage<D>, len: u64) -> Result<(), DbError> {
        self.data_index.len = len;
        storage.insert_at(self.storage_index, 0, &self.len())
    }

    fn set_state(
        &mut self,
        storage: &mut Storage<D>,
        index: u64,
        state: MapValueState,
    ) -> Result<(), DbError> {
        self.states.replace(storage, index, &state)?;
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

    fn state(&self, storage: &Storage<D>, index: u64) -> Result<MapValueState, DbError> {
        self.states.value(storage, index)
    }

    fn swap(&mut self, storage: &mut Storage<D>, index: u64, other: u64) -> Result<(), DbError> {
        self.states.swap(storage, index, other)?;
        self.keys.swap(storage, index, other)?;
        self.values.swap(storage, index, other)
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

impl<'a, K, T, D, Data> Iterator for MapIterator<'a, K, T, D, Data>
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
        self.multi_map.contains_value(storage, key, value)
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

    pub fn iter<'a>(&'a self, storage: &'a Storage<D>) -> MapIterator<K, T, D, Data> {
        self.multi_map.iter(storage)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> u64 {
        self.multi_map.len()
    }

    pub fn remove(&mut self, storage: &mut Storage<D>, key: &K) -> Result<(), DbError> {
        self.multi_map.remove_key(storage, key)
    }

    #[allow(dead_code)]
    pub fn reserve(&mut self, storage: &mut Storage<D>, capacity: u64) -> Result<(), DbError> {
        self.multi_map.reserve(storage, capacity)
    }

    pub fn value(&self, storage: &Storage<D>, key: &K) -> Result<Option<T>, DbError> {
        self.multi_map.value(storage, key)
    }
}

pub type DbMap<K, T, D> = MapImpl<K, T, D, DbMapData<K, T, D>>;

impl<K, T, D> DbMap<K, T, D>
where
    K: Default + Clone + VecValue,
    T: Default + Clone + VecValue,
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
    use crate::{
        storage::file_storage_memory_mapped::FileStorageMemoryMapped,
        test_utilities::test_file::TestFile,
    };

    #[test]
    fn derived_from_clone() {
        let state = MapValueState::Empty;
        let other = state.clone();
        assert_eq!(state, other);
    }

    #[test]
    fn contains_key() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();
        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        assert_eq!(map.contains(&storage, &1), Ok(false));

        map.insert(&mut storage, &1, &10).unwrap();

        assert_eq!(map.contains(&storage, &1), Ok(true));
    }

    #[test]
    fn contains_key_removed() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();
        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
        map.insert(&mut storage, &1, &10).unwrap();
        map.remove(&mut storage, &1).unwrap();

        assert_eq!(map.contains(&storage, &1), Ok(false));
    }

    #[test]
    fn contains_key_missing() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();
        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
        map.insert(&mut storage, &1, &10).unwrap();

        assert_eq!(map.contains(&storage, &2), Ok(false));
    }
    #[test]
    fn contains_value() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();
        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        assert_eq!(map.contains_value(&storage, &1, &10), Ok(false));

        map.insert(&mut storage, &1, &10).unwrap();

        assert_eq!(map.contains_value(&storage, &1, &10), Ok(true));
    }

    #[test]
    fn contains_value_removed() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();
        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
        map.insert(&mut storage, &1, &10).unwrap();
        map.remove(&mut storage, &1).unwrap();

        assert_eq!(map.contains_value(&storage, &1, &1), Ok(false));
    }

    #[test]
    fn contains_value_missing() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();
        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
        map.insert(&mut storage, &1, &10).unwrap();

        assert_eq!(map.contains_value(&storage, &1, &1), Ok(false));
    }

    #[test]
    fn from_storage_index() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let index;

        {
            let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
            map.insert(&mut storage, &1, &1).unwrap();
            map.insert(&mut storage, &3, &2).unwrap();
            map.insert(&mut storage, &5, &3).unwrap();
            map.remove(&mut storage, &3).unwrap();
            index = map.storage_index();
        }

        let map =
            DbMap::<u64, u64, FileStorageMemoryMapped>::from_storage(&storage, index).unwrap();
        let expected = vec![(1_u64, 1_u64), (5_u64, 3_u64)];

        assert_eq!(map.iter(&storage,).collect::<Vec<(u64, u64)>>(), expected);
    }

    #[test]
    fn from_storage_missing_index() {
        let test_file = TestFile::new();
        let storage = Storage::new(test_file.file_name()).unwrap();
        assert_eq!(
            DbMap::<u64, u64, FileStorageMemoryMapped>::from_storage(
                &storage,
                StorageIndex::from(1_u64)
            )
            .err()
            .unwrap(),
            DbError::from("Storage error: index (1) not found")
        );
    }

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        map.insert(&mut storage, &1, &10).unwrap();
        map.insert(&mut storage, &5, &15).unwrap();
        map.insert(&mut storage, &7, &20).unwrap();

        assert_eq!(map.len(), 3);
        assert_eq!(map.value(&storage, &1), Ok(Some(10)));
        assert_eq!(map.value(&storage, &5), Ok(Some(15)));
        assert_eq!(map.value(&storage, &7), Ok(Some(20)));
    }

    #[test]
    fn insert_reallocates() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        assert_eq!(map.capacity(), 0);

        for i in 0..100 {
            map.insert(&mut storage, &i, &i).unwrap();
        }

        assert_eq!(map.len(), 100);
        assert_eq!(map.capacity(), 128);

        for i in 0..100 {
            assert_eq!(map.value(&storage, &i), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_reallocates_with_collisions() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        for i in 1..100 {
            map.insert(&mut storage, &(i * 64 - 1), &i).unwrap();
        }

        for i in 1..100 {
            assert_eq!(map.value(&storage, &(i * 64 - 1)), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_same_key() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        assert_eq!(map.insert(&mut storage, &1, &10), Ok(None));
        assert_eq!(map.insert(&mut storage, &5, &15), Ok(None));
        assert_eq!(map.len(), 2);
        assert_eq!(map.insert(&mut storage, &5, &20), Ok(Some(15)));
        assert_eq!(map.len(), 2);

        assert_eq!(map.value(&storage, &1), Ok(Some(10)));
        assert_eq!(map.value(&storage, &5), Ok(Some(20)));
    }

    #[test]
    fn is_empty() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        assert!(map.is_empty());
        map.insert(&mut storage, &1, &10).unwrap();
        assert!(!map.is_empty());
        map.remove(&mut storage, &1).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn iter() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        map.insert(&mut storage, &1, &10).unwrap();
        map.insert(&mut storage, &5, &15).unwrap();
        map.insert(&mut storage, &7, &20).unwrap();
        map.insert(&mut storage, &2, &30).unwrap();
        map.insert(&mut storage, &4, &13).unwrap();
        map.remove(&mut storage, &7).unwrap();

        let mut actual = map.iter(&storage).collect::<Vec<(u64, u64)>>();
        actual.sort();
        let expected: Vec<(u64, u64)> = vec![(1, 10), (2, 30), (4, 13), (5, 15)];

        assert_eq!(actual, expected);
    }

    #[test]
    fn remove() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        map.insert(&mut storage, &1, &10).unwrap();
        map.insert(&mut storage, &5, &15).unwrap();
        map.insert(&mut storage, &7, &20).unwrap();

        assert_eq!(map.len(), 3);
        map.remove(&mut storage, &5).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(map.value(&storage, &1), Ok(Some(10)));
        assert_eq!(map.value(&storage, &5), Ok(None));
        assert_eq!(map.value(&storage, &7), Ok(Some(20)));
    }

    #[test]
    fn remove_deleted() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        map.insert(&mut storage, &1, &10).unwrap();
        map.insert(&mut storage, &5, &15).unwrap();
        map.insert(&mut storage, &7, &20).unwrap();

        assert_eq!(map.len(), 3);

        map.remove(&mut storage, &5).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(map.value(&storage, &5), Ok(None));

        map.remove(&mut storage, &5).unwrap();

        assert_eq!(map.len(), 2);
    }

    #[test]
    fn remove_missing() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        assert_eq!(map.len(), 0);
        assert_eq!(map.remove(&mut storage, &0), Ok(()));

        map.insert(&mut storage, &1, &10).unwrap();

        assert_eq!(map.len(), 1);

        map.remove(&mut storage, &0).unwrap();

        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove_shrinks_capacity() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        for i in 0..100 {
            map.insert(&mut storage, &i, &i).unwrap();
        }

        assert_eq!(map.len(), 100);
        assert_eq!(map.capacity(), 128);

        for i in (0..100).rev() {
            map.remove(&mut storage, &i).unwrap();
        }

        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), 64);
    }

    #[test]
    fn reserve_larger() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
        map.insert(&mut storage, &1, &1).unwrap();

        let capacity = map.capacity() + 10;
        let size = map.len();

        map.reserve(&mut storage, capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.len(), size);
        assert_eq!(map.value(&storage, &1), Ok(Some(1)));
    }

    #[test]
    fn reserve_same() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
        map.insert(&mut storage, &1, &1).unwrap();

        let capacity = map.capacity();
        let size = map.len();

        map.reserve(&mut storage, capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.len(), size);
    }

    #[test]
    fn reserve_smaller() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();
        map.insert(&mut storage, &1, &1).unwrap();

        let current_capacity = map.capacity();
        let capacity = current_capacity - 10;
        let size = map.len();

        map.reserve(&mut storage, capacity).unwrap();

        assert_eq!(map.capacity(), current_capacity);
        assert_eq!(map.len(), size);
    }

    #[test]
    fn value_missing() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        assert_eq!(map.value(&storage, &0), Ok(None));
    }

    #[test]
    fn values_at_end() {
        let test_file = TestFile::new();
        let mut storage = Storage::new(test_file.file_name()).unwrap();

        let mut map = DbMap::<u64, u64, FileStorageMemoryMapped>::new(&mut storage).unwrap();

        map.insert(&mut storage, &127, &10).unwrap();
        map.insert(&mut storage, &255, &11).unwrap();
        map.insert(&mut storage, &191, &12).unwrap();

        assert_eq!(map.value(&storage, &127), Ok(Some(10)));
        assert_eq!(map.value(&storage, &255), Ok(Some(11)));
        assert_eq!(map.value(&storage, &191), Ok(Some(12)));
    }

    #[test]
    fn bad_deserialize() {
        assert_eq!(
            MapDataIndex::deserialize(&Vec::<u8>::new()).err().unwrap(),
            DbError::from("MapDataStorageIndex deserialization error: not enough data")
        );
    }

    #[test]
    fn derived_from_debug() {
        let value = MapValueState::Deleted;
        format!("{value:?}");
    }
    #[test]
    fn derived_from_default() {
        assert_eq!(MapValueState::default(), MapValueState::Empty);
    }

    #[test]
    fn map_value_state_bad_deserialize() {
        assert_eq!(
            MapValueState::deserialize(&Vec::<u8>::new()).err().unwrap(),
            DbError::from("MapValueState deserialization error: unknown value")
        );
    }

    #[test]
    fn serialized_size() {
        assert_eq!(MapValueState::default().serialized_size(), 1);
    }
}
