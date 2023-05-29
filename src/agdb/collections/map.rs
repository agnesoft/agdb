use super::multi_map::MultiMapImpl;
use super::vec::DbVec;
use crate::collections::vec::VecValue;
use crate::db::db_error::DbError;
use crate::storage::file_storage::FileStorage;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::stable_hash::StableHash;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use std::rc::Rc;

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
    fn storage_len() -> u64 {
        Self::serialized_size_static()
    }
}

pub trait MapData<K, T>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + PartialEq,
{
    fn capacity(&self) -> u64;
    fn commit(&mut self, id: u64) -> Result<(), DbError>;
    fn len(&self) -> u64;
    fn key(&self, index: u64) -> Result<K, DbError>;
    fn resize(&mut self, capacity: u64) -> Result<(), DbError>;
    fn set_len(&mut self, len: u64) -> Result<(), DbError>;
    fn set_state(&mut self, index: u64, state: MapValueState) -> Result<(), DbError>;
    fn set_key(&mut self, index: u64, key: &K) -> Result<(), DbError>;
    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError>;
    fn state(&self, index: u64) -> Result<MapValueState, DbError>;
    fn swap(&mut self, index: u64, other: u64) -> Result<(), DbError>;
    fn transaction(&mut self) -> u64;
    fn value(&self, index: u64) -> Result<T, DbError>;
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

pub struct DbMapData<K, T, S = FileStorage>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + VecValue,
    T: Clone + Default + Eq + PartialEq + VecValue,
    S: Storage,
{
    storage: Rc<RefCell<S>>,
    storage_index: StorageIndex,
    data_index: MapDataIndex,
    states: DbVec<MapValueState, S>,
    keys: DbVec<K, S>,
    values: DbVec<T, S>,
}

impl<K, T, S> DbMapData<K, T, S>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + VecValue,
    T: Clone + Default + Eq + PartialEq + VecValue,
    S: Storage,
{
    pub fn new(storage: Rc<RefCell<S>>) -> Result<Self, DbError> {
        let states = DbVec::<MapValueState, S>::new(storage.clone())?;
        let keys = DbVec::<K, S>::new(storage.clone())?;
        let values = DbVec::<T, S>::new(storage.clone())?;

        let data_index = MapDataIndex {
            len: 0,
            states_index: states.storage_index(),
            keys_index: keys.storage_index(),
            values_index: values.storage_index(),
        };

        let storage_index = storage.borrow_mut().insert(&data_index)?;

        Ok(Self {
            storage,
            storage_index,
            data_index,
            states,
            keys,
            values,
        })
    }

    pub fn from_storage(
        storage: Rc<RefCell<S>>,
        storage_index: StorageIndex,
    ) -> Result<Self, DbError> {
        let data_index = storage.borrow_mut().value::<MapDataIndex>(storage_index)?;
        let states =
            DbVec::<MapValueState, S>::from_storage(storage.clone(), data_index.states_index)?;
        let keys = DbVec::<K, S>::from_storage(storage.clone(), data_index.keys_index)?;
        let values = DbVec::<T, S>::from_storage(storage.clone(), data_index.values_index)?;

        Ok(Self {
            storage,
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

impl<K, T, S> MapData<K, T> for DbMapData<K, T, S>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + VecValue,
    T: Clone + Default + Eq + PartialEq + VecValue,
    S: Storage,
{
    fn capacity(&self) -> u64 {
        self.states.len()
    }

    fn commit(&mut self, id: u64) -> Result<(), DbError> {
        self.storage.borrow_mut().commit(id)
    }

    fn len(&self) -> u64 {
        self.data_index.len
    }

    fn key(&self, index: u64) -> Result<K, DbError> {
        self.keys.value(index)
    }

    fn resize(&mut self, capacity: u64) -> Result<(), DbError> {
        self.states.resize(capacity, &MapValueState::Empty)?;
        self.keys.resize(capacity, &K::default())?;
        self.values.resize(capacity, &T::default())
    }

    fn set_len(&mut self, len: u64) -> Result<(), DbError> {
        self.data_index.len = len;
        self.storage
            .borrow_mut()
            .insert_at(self.storage_index, 0, &self.len())
    }

    fn set_state(&mut self, index: u64, state: MapValueState) -> Result<(), DbError> {
        self.states.replace(index, &state)?;
        Ok(())
    }

    fn set_key(&mut self, index: u64, key: &K) -> Result<(), DbError> {
        self.keys.replace(index, key)?;
        Ok(())
    }

    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        self.values.replace(index, value)?;
        Ok(())
    }

    fn state(&self, index: u64) -> Result<MapValueState, DbError> {
        self.states.value(index)
    }

    fn swap(&mut self, index: u64, other: u64) -> Result<(), DbError> {
        self.states.swap(index, other)?;
        self.keys.swap(index, other)?;
        self.values.swap(index, other)
    }

    fn transaction(&mut self) -> u64 {
        self.storage.borrow_mut().transaction()
    }

    fn value(&self, index: u64) -> Result<T, DbError> {
        self.values.value(index)
    }
}

pub struct MapIterator<'a, K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + PartialEq,
    Data: MapData<K, T>,
{
    pub pos: u64,
    pub data: &'a Data,
    pub phantom_data: PhantomData<(K, T)>,
}

impl<'a, K, T, Data> Iterator for MapIterator<'a, K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + PartialEq,
    Data: MapData<K, T>,
{
    type Item = (K, T);

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos != self.data.capacity() {
            let current_pos = self.pos;
            self.pos += 1;

            if self.data.state(current_pos).unwrap_or_default() == MapValueState::Valid {
                let key = self.data.key(current_pos).unwrap_or_default();
                let value = self.data.value(current_pos).unwrap_or_default();

                return Some((key, value));
            }
        }

        None
    }
}

pub struct MapImpl<K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + PartialEq,
    Data: MapData<K, T>,
{
    pub(crate) multi_map: MultiMapImpl<K, T, Data>,
}

impl<K, T, Data> MapImpl<K, T, Data>
where
    K: Default + Eq + Hash + PartialEq + StableHash,
    T: Default + Eq + PartialEq,
    Data: MapData<K, T>,
{
    #[allow(dead_code)]
    pub fn capacity(&self) -> u64 {
        self.multi_map.capacity()
    }

    #[allow(dead_code)]
    pub fn contains(&self, key: &K) -> Result<bool, DbError> {
        self.multi_map.contains(key)
    }

    #[allow(dead_code)]
    pub fn contains_value(&self, key: &K, value: &T) -> Result<bool, DbError> {
        self.multi_map.contains_value(key, value)
    }

    pub fn insert(&mut self, key: &K, value: &T) -> Result<Option<T>, DbError> {
        let old_value = self.value(key)?;

        if let Some(old) = &old_value {
            self.multi_map.replace(key, old, value)?;
        } else {
            self.multi_map.insert(key, value)?;
        }

        Ok(old_value)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.multi_map.is_empty()
    }

    pub fn iter(&self) -> MapIterator<K, T, Data> {
        self.multi_map.iter()
    }

    pub fn len(&self) -> u64 {
        self.multi_map.len()
    }

    pub fn remove(&mut self, key: &K) -> Result<(), DbError> {
        self.multi_map.remove_key(key)
    }

    #[allow(dead_code)]
    pub fn reserve(&mut self, capacity: u64) -> Result<(), DbError> {
        self.multi_map.reserve(capacity)
    }

    pub fn value(&self, key: &K) -> Result<Option<T>, DbError> {
        self.multi_map.value(key)
    }
}

pub type DbMap<K, T, S = FileStorage> = MapImpl<K, T, DbMapData<K, T, S>>;

impl<K, T, S> DbMap<K, T, S>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + VecValue,
    T: Clone + Default + Eq + PartialEq + VecValue,
    S: Storage,
{
    pub fn new(storage: Rc<RefCell<S>>) -> Result<Self, DbError> {
        Ok(Self {
            multi_map: MultiMapImpl::<K, T, DbMapData<K, T, S>> {
                data: DbMapData::<K, T, S>::new(storage)?,
                phantom_marker: PhantomData,
            },
        })
    }

    pub fn from_storage(storage: Rc<RefCell<S>>, index: StorageIndex) -> Result<Self, DbError> {
        Ok(Self {
            multi_map: MultiMapImpl::<K, T, DbMapData<K, T, S>> {
                data: DbMapData::<K, T, S>::from_storage(storage, index)?,
                phantom_marker: PhantomData,
            },
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.multi_map.data.storage_index()
    }

    #[allow(dead_code)]
    pub fn to_hash_map(&self) -> HashMap<K, T> {
        let mut map = HashMap::<K, T>::new();
        map.reserve(self.len() as usize);

        for (key, value) in self.iter() {
            map.insert(key, value);
        }

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;
    use std::collections::HashMap;

    #[test]
    fn contains_key() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        assert_eq!(map.contains(&1), Ok(false));

        map.insert(&1, &10).unwrap();

        assert_eq!(map.contains(&1), Ok(true));
    }

    #[test]
    fn contains_key_removed() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = DbMap::<u64, u64>::new(storage).unwrap();
        map.insert(&1, &10).unwrap();
        map.remove(&1).unwrap();

        assert_eq!(map.contains(&1), Ok(false));
    }

    #[test]
    fn contains_key_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = DbMap::<u64, u64>::new(storage).unwrap();
        map.insert(&1, &10).unwrap();

        assert_eq!(map.contains(&2), Ok(false));
    }
    #[test]
    fn contains_value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        assert_eq!(map.contains_value(&1, &10), Ok(false));

        map.insert(&1, &10).unwrap();

        assert_eq!(map.contains_value(&1, &10), Ok(true));
    }

    #[test]
    fn contains_value_removed() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = DbMap::<u64, u64>::new(storage).unwrap();
        map.insert(&1, &10).unwrap();
        map.remove(&1).unwrap();

        assert_eq!(map.contains_value(&1, &1), Ok(false));
    }

    #[test]
    fn contains_value_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut map = DbMap::<u64, u64>::new(storage).unwrap();
        map.insert(&1, &10).unwrap();

        assert_eq!(map.contains_value(&1, &1), Ok(false));
    }

    #[test]
    fn from_storage_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let index;

        {
            let mut map = DbMap::<u64, u64>::new(storage.clone()).unwrap();
            map.insert(&1, &1).unwrap();
            map.insert(&3, &2).unwrap();
            map.insert(&5, &3).unwrap();
            map.remove(&3).unwrap();
            index = map.storage_index();
        }

        let map = DbMap::<u64, u64>::from_storage(storage, index).unwrap();

        let mut expected = HashMap::<u64, u64>::new();
        expected.insert(1, 1);
        expected.insert(5, 3);

        assert_eq!(map.to_hash_map(), expected);
    }

    #[test]
    fn from_storage_missing_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        assert_eq!(
            DbMap::<u64, u64>::from_storage(storage, StorageIndex::from(1_u64))
                .err()
                .unwrap(),
            DbError::from("FileStorage error: index (1) not found")
        );
    }

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();

        assert_eq!(map.len(), 3);
        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(15)));
        assert_eq!(map.value(&7), Ok(Some(20)));
    }

    #[test]
    fn insert_reallocates() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        assert_eq!(map.capacity(), 0);

        for i in 0..100 {
            map.insert(&i, &i).unwrap();
        }

        assert_eq!(map.len(), 100);
        assert_eq!(map.capacity(), 128);

        for i in 0..100 {
            assert_eq!(map.value(&i), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_reallocates_with_collisions() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        for i in 1..100 {
            map.insert(&(i * 64 - 1), &i).unwrap();
        }

        for i in 1..100 {
            assert_eq!(map.value(&(i * 64 - 1)), Ok(Some(i)));
        }
    }

    #[test]
    fn insert_same_key() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        assert_eq!(map.insert(&1, &10), Ok(None));
        assert_eq!(map.insert(&5, &15), Ok(None));
        assert_eq!(map.len(), 2);
        assert_eq!(map.insert(&5, &20), Ok(Some(15)));
        assert_eq!(map.len(), 2);

        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(Some(20)));
    }

    #[test]
    fn is_empty() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        assert!(map.is_empty());
        map.insert(&1, &10).unwrap();
        assert!(!map.is_empty());
        map.remove(&1).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn iter() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();
        map.insert(&2, &30).unwrap();
        map.insert(&4, &13).unwrap();
        map.remove(&7).unwrap();

        let mut actual = map.iter().collect::<Vec<(u64, u64)>>();
        actual.sort();
        let expected: Vec<(u64, u64)> = vec![(1, 10), (2, 30), (4, 13), (5, 15)];

        assert_eq!(actual, expected);
    }

    #[test]
    fn remove() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();

        assert_eq!(map.len(), 3);
        map.remove(&5).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(map.value(&1), Ok(Some(10)));
        assert_eq!(map.value(&5), Ok(None));
        assert_eq!(map.value(&7), Ok(Some(20)));
    }

    #[test]
    fn remove_deleted() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();

        assert_eq!(map.len(), 3);

        map.remove(&5).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(map.value(&5), Ok(None));

        map.remove(&5).unwrap();

        assert_eq!(map.len(), 2);
    }

    #[test]
    fn remove_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        assert_eq!(map.len(), 0);
        assert_eq!(map.remove(&0), Ok(()));

        map.insert(&1, &10).unwrap();

        assert_eq!(map.len(), 1);

        map.remove(&0).unwrap();

        assert_eq!(map.len(), 1);
    }

    #[test]
    fn remove_shrinks_capacity() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        for i in 0..100 {
            map.insert(&i, &i).unwrap();
        }

        assert_eq!(map.len(), 100);
        assert_eq!(map.capacity(), 128);

        for i in (0..100).rev() {
            map.remove(&i).unwrap();
        }

        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), 64);
    }

    #[test]
    fn reserve_larger() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();
        map.insert(&1, &1).unwrap();

        let capacity = map.capacity() + 10;
        let size = map.len();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.len(), size);
        assert_eq!(map.value(&1), Ok(Some(1)));
    }

    #[test]
    fn reserve_same() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();
        map.insert(&1, &1).unwrap();

        let capacity = map.capacity();
        let size = map.len();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), capacity);
        assert_eq!(map.len(), size);
    }

    #[test]
    fn reserve_smaller() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();
        map.insert(&1, &1).unwrap();

        let current_capacity = map.capacity();
        let capacity = current_capacity - 10;
        let size = map.len();

        map.reserve(capacity).unwrap();

        assert_eq!(map.capacity(), current_capacity);
        assert_eq!(map.len(), size);
    }

    #[test]
    fn to_hash_map() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();
        map.insert(&1, &10).unwrap();
        map.insert(&5, &15).unwrap();
        map.insert(&7, &20).unwrap();
        map.remove(&5).unwrap();

        let other = map.to_hash_map();

        assert_eq!(other.len(), 2);
        assert_eq!(other.get(&1), Some(&10));
        assert_eq!(other.get(&5), None);
        assert_eq!(other.get(&7), Some(&20));
    }

    #[test]
    fn to_hash_map_empty() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let map = DbMap::<u64, u64>::new(storage).unwrap();
        let other = map.to_hash_map();

        assert_eq!(other.len(), 0);
    }

    #[test]
    fn value_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let map = DbMap::<u64, u64>::new(storage).unwrap();

        assert_eq!(map.value(&0), Ok(None));
    }

    #[test]
    fn values_at_end() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let mut map = DbMap::<u64, u64>::new(storage).unwrap();

        map.insert(&127, &10).unwrap();
        map.insert(&255, &11).unwrap();
        map.insert(&191, &12).unwrap();

        assert_eq!(map.value(&127), Ok(Some(10)));
        assert_eq!(map.value(&255), Ok(Some(11)));
        assert_eq!(map.value(&191), Ok(Some(12)));
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
