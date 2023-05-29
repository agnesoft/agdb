use super::multi_map::MultiMapStorage;
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
use std::marker::PhantomData;
use std::rc::Rc;

pub trait DictionaryData<T>
where
    T: Clone + Default + Eq + PartialEq + StableHash + VecValue,
{
    fn capacity(&self) -> u64;
    fn commit(&mut self, id: u64) -> Result<(), DbError>;
    fn count(&self, index: u64) -> Result<u64, DbError>;
    fn hash(&self, index: u64) -> Result<u64, DbError>;
    fn indexes(&self, hash: u64) -> Result<Vec<u64>, DbError>;
    fn insert(&mut self, hash: u64, index: u64) -> Result<(), DbError>;
    fn remove(&mut self, hash: u64, index: u64) -> Result<(), DbError>;
    fn set_capacity(&mut self, capacity: u64) -> Result<(), DbError>;
    fn set_count(&mut self, index: u64, count: u64) -> Result<(), DbError>;
    fn set_hash(&mut self, index: u64, hash: u64) -> Result<(), DbError>;
    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError>;
    fn transaction(&mut self) -> u64;
    fn value(&self, index: u64) -> Result<T, DbError>;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DictionaryIndex(pub u64);

pub struct DictionaryDataStorage<T, Data = FileStorage>
where
    T: Clone + Default + Eq + PartialEq + StableHash + VecValue,
    Data: Storage,
{
    storage: Rc<RefCell<Data>>,
    storage_index: StorageIndex,
    index: MultiMapStorage<u64, u64, Data>,
    counts: DbVec<u64, Data>,
    hashes: DbVec<u64, Data>,
    values: DbVec<T, Data>,
}

struct DictionaryDataStorageIndexes {
    index_index: StorageIndex,
    counts_index: StorageIndex,
    hashes_index: StorageIndex,
    values_index: StorageIndex,
}

impl SerializeStatic for DictionaryDataStorageIndexes {
    fn serialized_size_static() -> u64 {
        StorageIndex::serialized_size_static() * 4
    }
}

impl<T, Data> DictionaryDataStorage<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + VecValue,
    Data: Storage,
{
    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        let index = MultiMapStorage::<u64, u64, Data>::new(storage.clone())?;
        let mut counts = DbVec::<u64, Data>::new(storage.clone())?;
        let mut hashes = DbVec::<u64, Data>::new(storage.clone())?;
        let mut values = DbVec::<T, Data>::new(storage.clone())?;

        let id = storage.borrow_mut().transaction();

        counts.push(&0)?;
        hashes.push(&0)?;
        values.push(&T::default())?;

        let data_index = DictionaryDataStorageIndexes {
            index_index: index.storage_index(),
            counts_index: counts.storage_index(),
            hashes_index: hashes.storage_index(),
            values_index: values.storage_index(),
        };

        let storage_index = storage.borrow_mut().insert(&data_index)?;

        storage.borrow_mut().commit(id)?;

        Ok(Self {
            storage,
            storage_index,
            index,
            counts,
            hashes,
            values,
        })
    }

    pub fn from_storage(
        storage: Rc<RefCell<Data>>,
        storage_index: StorageIndex,
    ) -> Result<Self, DbError> {
        let data_index = storage
            .borrow_mut()
            .value::<DictionaryDataStorageIndexes>(storage_index)?;
        let index = MultiMapStorage::<u64, u64, Data>::from_storage(
            storage.clone(),
            data_index.index_index,
        )?;
        let counts = DbVec::<u64, Data>::from_storage(storage.clone(), data_index.counts_index)?;
        let hashes = DbVec::<u64, Data>::from_storage(storage.clone(), data_index.hashes_index)?;
        let values = DbVec::<T, Data>::from_storage(storage.clone(), data_index.values_index)?;

        Ok(Self {
            storage,
            storage_index,
            index,
            counts,
            hashes,
            values,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.storage_index
    }
}

impl<T, Data> DictionaryData<T> for DictionaryDataStorage<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + VecValue,
    Data: Storage,
{
    fn capacity(&self) -> u64 {
        self.counts.len()
    }

    fn commit(&mut self, id: u64) -> Result<(), DbError> {
        self.storage.borrow_mut().commit(id)
    }

    fn count(&self, index: u64) -> Result<u64, DbError> {
        self.counts.value(index)
    }

    fn indexes(&self, hash: u64) -> Result<Vec<u64>, DbError> {
        self.index.values(&hash)
    }

    fn insert(&mut self, hash: u64, index: u64) -> Result<(), DbError> {
        self.index.insert(&hash, &index)
    }

    fn hash(&self, index: u64) -> Result<u64, DbError> {
        self.hashes.value(index)
    }

    fn remove(&mut self, hash: u64, index: u64) -> Result<(), DbError> {
        self.index.remove_value(&hash, &index)
    }

    fn set_capacity(&mut self, capacity: u64) -> Result<(), DbError> {
        self.counts.resize(capacity, &0)?;
        self.hashes.resize(capacity, &0)?;
        self.values.resize(capacity, &T::default())
    }

    fn set_count(&mut self, index: u64, count: u64) -> Result<(), DbError> {
        self.counts.replace(index, &count)?;
        Ok(())
    }

    fn set_hash(&mut self, index: u64, hash: u64) -> Result<(), DbError> {
        self.hashes.replace(index, &hash)?;
        Ok(())
    }

    fn set_value(&mut self, index: u64, value: &T) -> Result<(), DbError> {
        self.values.replace(index, value)?;
        Ok(())
    }

    fn transaction(&mut self) -> u64 {
        self.storage.borrow_mut().transaction()
    }

    fn value(&self, index: u64) -> Result<T, DbError> {
        self.values.value(index)
    }
}

impl Serialize for DictionaryDataStorageIndexes {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(self.serialized_size() as usize);
        bytes.extend(self.index_index.serialize());
        bytes.extend(self.counts_index.serialize());
        bytes.extend(self.hashes_index.serialize());
        bytes.extend(self.values_index.serialize());

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        if bytes.len() < Self::serialized_size_static() as usize {
            return Err(DbError::from(
                "DictionaryDataStorageIndexes deserialization error: not enough data",
            ));
        }

        Ok(DictionaryDataStorageIndexes {
            index_index: StorageIndex::deserialize(bytes)?,
            counts_index: StorageIndex::deserialize(
                &bytes[StorageIndex::serialized_size_static() as usize..],
            )?,
            hashes_index: StorageIndex::deserialize(
                &bytes[(StorageIndex::serialized_size_static() * 2) as usize..],
            )?,
            values_index: StorageIndex::deserialize(
                &bytes[(StorageIndex::serialized_size_static() * 3) as usize..],
            )?,
        })
    }

    fn serialized_size(&self) -> u64 {
        Self::serialized_size_static()
    }
}

pub struct DictionaryImpl<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + VecValue,
    Data: DictionaryData<T>,
{
    pub data: Data,
    pub phantom_data: PhantomData<T>,
}

impl<T, Data> DictionaryImpl<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + VecValue,
    Data: DictionaryData<T>,
{
    #[allow(dead_code)]
    pub fn count(&self, index: DictionaryIndex) -> Result<Option<u64>, DbError> {
        if self.is_valid_index(index.0) {
            let count = self.data.count(index.0)?;

            if count != 0 {
                return Ok(Some(count));
            }
        }

        Ok(None)
    }

    pub fn len(&self) -> Result<u64, DbError> {
        self.data.count(0)
    }

    #[allow(dead_code)]
    pub fn index(&self, value: &T) -> Result<Option<DictionaryIndex>, DbError> {
        self.find_value(value.stable_hash(), value)
    }

    pub fn insert(&mut self, value: &T) -> Result<DictionaryIndex, DbError> {
        let hash = value.stable_hash();
        let index;

        let id = self.data.transaction();

        if let Some(i) = self.find_value(hash, value)? {
            index = i;
            let count = self.data.count(index.0)?;
            self.data.set_count(index.0, count + 1)?;
        } else {
            index = DictionaryIndex(self.insert_new(hash, value)?);
        }

        self.data.commit(id)?;

        Ok(index)
    }

    pub fn remove(&mut self, index: DictionaryIndex) -> Result<bool, DbError> {
        if self.is_valid_index(index.0) {
            let count = self.data.count(index.0)?;

            if count != 0 {
                let id = self.data.transaction();

                if count == 1 {
                    self.remove_value(index.0)?
                } else {
                    self.data.set_count(index.0, count - 1)?
                }

                self.data.commit(id)?;
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn value(&self, index: DictionaryIndex) -> Result<Option<T>, DbError> {
        let mut v = None;

        if self.is_valid_index(index.0) && self.data.count(index.0)? != 0 {
            v = Some(self.data.value(index.0)?);
        }

        Ok(v)
    }

    fn find_value(&self, hash: u64, value: &T) -> Result<Option<DictionaryIndex>, DbError> {
        for index in self.data.indexes(hash)? {
            if *value == self.data.value(index)? {
                return Ok(Some(DictionaryIndex(index)));
            }
        }

        Ok(None)
    }

    fn free_index(&mut self, index: u64) -> Result<(), DbError> {
        let next_free_index = self.data.hash(0)?;
        self.data.set_hash(index, next_free_index)?;
        self.data.set_hash(0, index)
    }

    fn get_free_index(&mut self) -> Result<u64, DbError> {
        let mut free_index = self.data.hash(0)?;

        if free_index == 0 {
            free_index = self.data.capacity();
            self.data.set_capacity(free_index + 1)?;
        } else {
            let next_free_index = self.data.hash(free_index)?;
            self.data.set_hash(0, next_free_index)?;
        }

        Ok(free_index)
    }

    fn insert_new(&mut self, hash: u64, value: &T) -> Result<u64, DbError> {
        let index = self.get_free_index()?;

        self.data.insert(hash, index)?;
        self.data.set_hash(index, hash)?;
        self.data.set_count(index, 1)?;
        self.data.set_value(index, value)?;

        let len = self.len()?;
        self.data.set_count(0, len + 1)?;

        Ok(index)
    }

    fn is_valid_index(&self, index: u64) -> bool {
        index != 0 && index < self.data.capacity()
    }

    fn remove_value(&mut self, index: u64) -> Result<(), DbError> {
        let hash = self.data.hash(index)?;
        self.data.remove(hash, index)?;
        self.free_index(index)?;
        self.data.set_count(index, 0)?;
        let len = self.len()?;
        self.data.set_count(0, len - 1)
    }
}

pub type DictionaryStorage<T, Data = FileStorage> =
    DictionaryImpl<T, DictionaryDataStorage<T, Data>>;

impl<T, Data> DictionaryStorage<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + VecValue,
    Data: Storage,
{
    pub fn new(storage: Rc<RefCell<Data>>) -> Result<Self, DbError> {
        Ok(Self {
            data: DictionaryDataStorage::new(storage)?,
            phantom_data: PhantomData,
        })
    }

    pub fn from_storage(storage: Rc<RefCell<Data>>, index: StorageIndex) -> Result<Self, DbError> {
        Ok(Self {
            data: DictionaryDataStorage::from_storage(storage, index)?,
            phantom_data: PhantomData,
        })
    }

    pub fn storage_index(&self) -> StorageIndex {
        self.data.storage_index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::collision_value::CollisionValue;
    use crate::test_utilities::test_file::TestFile;
    use std::cmp::Ordering;

    #[test]
    fn count_invalid_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        assert_eq!(dictionary.count(DictionaryIndex(u64::MAX)), Ok(None));
    }

    #[test]
    fn index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.index(&10), Ok(Some(index)));
    }

    #[test]
    fn index_missing_value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        assert_eq!(dictionary.index(&10), Ok(None));
    }

    #[test]
    fn index_removed_value() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();
        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.index(&10), Ok(None));
    }

    #[test]
    fn index_reuse() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index1 = dictionary.insert(&5).unwrap();
        let index2 = dictionary.insert(&10).unwrap();
        let index3 = dictionary.insert(&7).unwrap();

        dictionary.remove(index2).unwrap();
        dictionary.remove(index1).unwrap();
        dictionary.remove(index3).unwrap();

        assert_eq!(dictionary.count(index1), Ok(None));
        assert_eq!(dictionary.count(index2), Ok(None));
        assert_eq!(dictionary.count(index3), Ok(None));

        assert_eq!(dictionary.insert(&3), Ok(index3));
        assert_eq!(dictionary.insert(&2), Ok(index1));
        assert_eq!(dictionary.insert(&1), Ok(index2));

        assert_eq!(dictionary.value(index1), Ok(Some(2)));
        assert_eq!(dictionary.value(index2), Ok(Some(1)));
        assert_eq!(dictionary.value(index3), Ok(Some(3)));
    }

    #[test]
    fn index_with_collisions() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<CollisionValue<i64>>::new(storage).unwrap();

        let index1 = dictionary.insert(&CollisionValue::new(1)).unwrap();
        let index2 = dictionary.insert(&CollisionValue::new(2)).unwrap();
        let index3 = dictionary.insert(&CollisionValue::new(3)).unwrap();

        assert_eq!(dictionary.index(&CollisionValue::new(1)), Ok(Some(index1)));
        assert_eq!(dictionary.index(&CollisionValue::new(2)), Ok(Some(index2)));
        assert_eq!(dictionary.index(&CollisionValue::new(3)), Ok(Some(index3)));
    }

    #[test]
    fn insert() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.len(), Ok(1));
        assert_eq!(dictionary.value(index), Ok(Some(10_i64)));
        assert_eq!(dictionary.count(index), Ok(Some(1)));
    }

    #[test]
    fn insert_multiple() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index1 = dictionary.insert(&10).unwrap();
        let index2 = dictionary.insert(&15).unwrap();
        let index3 = dictionary.insert(&20).unwrap();

        assert_eq!(dictionary.len(), Ok(3));

        assert_eq!(dictionary.value(index1).unwrap(), Some(10_i64));
        assert_eq!(dictionary.count(index1), Ok(Some(1)));

        assert_eq!(dictionary.value(index2).unwrap(), Some(15_i64));
        assert_eq!(dictionary.count(index2), Ok(Some(1)));

        assert_eq!(dictionary.value(index3).unwrap(), Some(20_i64));
        assert_eq!(dictionary.count(index3), Ok(Some(1)));
    }

    #[test]
    fn insert_same() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        dictionary.insert(&10).unwrap();

        let index2 = dictionary.insert(&15).unwrap();

        assert_eq!(dictionary.insert(&15).unwrap(), index2);
        assert_eq!(dictionary.insert(&15).unwrap(), index2);

        dictionary.insert(&20).unwrap();

        assert_eq!(dictionary.len(), Ok(3));
        assert_eq!(dictionary.count(index2), Ok(Some(3)));
    }

    #[test]
    fn remove() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();
        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.value(index), Ok(None));
        assert_eq!(dictionary.count(index), Ok(None));

        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.value(index), Ok(None));
        assert_eq!(dictionary.count(index), Ok(None));
    }

    #[test]
    fn remove_duplicated() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();
        dictionary.insert(&10).unwrap();
        dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.value(index), Ok(Some(10)));
        assert_eq!(dictionary.count(index), Ok(Some(3)));

        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.value(index), Ok(Some(10)));
        assert_eq!(dictionary.count(index), Ok(Some(2)));

        dictionary.remove(index).unwrap();
        dictionary.remove(index).unwrap();

        assert_eq!(dictionary.value(index), Ok(None));
        assert_eq!(dictionary.count(index), Ok(None));
    }

    #[test]
    fn remove_missing() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let mut dictionary = DictionaryStorage::<i64>::new(storage).unwrap();

        let index = dictionary.insert(&10).unwrap();

        assert_eq!(dictionary.len(), Ok(1));

        dictionary.remove(DictionaryIndex(index.0 + 1)).unwrap();

        assert_eq!(dictionary.len(), Ok(1));
    }

    #[test]
    fn restore_from_file() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));

        let storage_index;
        let index1;
        let index2;
        let index3;
        let index4;

        {
            let mut dictionary = DictionaryStorage::<i64>::new(storage.clone()).unwrap();
            storage_index = dictionary.storage_index();

            index1 = dictionary.insert(&10).unwrap();
            dictionary.insert(&10).unwrap();
            index2 = dictionary.insert(&15).unwrap();
            index3 = dictionary.insert(&7).unwrap();
            index4 = dictionary.insert(&20).unwrap();
            dictionary.remove(index2).unwrap();
        }

        let dictionary = DictionaryStorage::<i64>::from_storage(storage, storage_index).unwrap();

        assert_eq!(dictionary.len(), Ok(3));
        assert_eq!(dictionary.count(index1), Ok(Some(2)));
        assert_eq!(dictionary.value(index1), Ok(Some(10)));
        assert_eq!(dictionary.value(index2), Ok(None));
        assert_eq!(dictionary.value(index3), Ok(Some(7)));
        assert_eq!(dictionary.value(index4), Ok(Some(20)));
    }

    #[test]
    fn value_missing_index() {
        let test_file = TestFile::new();
        let storage = Rc::new(RefCell::new(
            FileStorage::new(test_file.file_name()).unwrap(),
        ));
        let dictionary = DictionaryStorage::<i64>::new(storage).unwrap();
        assert_eq!(dictionary.value(DictionaryIndex(1)), Ok(None));
    }

    #[test]
    fn bad_deserialize() {
        assert_eq!(
            DictionaryDataStorageIndexes::deserialize(&Vec::<u8>::new())
                .err()
                .unwrap(),
            DbError::from("DictionaryDataStorageIndexes deserialization error: not enough data")
        );
    }

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn derived_from_clone() {
        let _ = DictionaryIndex(0).clone();
    }

    #[test]
    fn derived_from_debug() {
        let _ = format!("{:?}", DictionaryIndex(0));
    }

    #[test]
    fn derived_from_default() {
        let _ = DictionaryIndex::default();
    }

    #[test]
    fn derived_from_ord() {
        assert_eq!(DictionaryIndex(1).cmp(&DictionaryIndex(1)), Ordering::Equal);
    }

    #[test]
    fn derived_from_partial_ord() {
        let mut ids = vec![DictionaryIndex(3), DictionaryIndex(0), DictionaryIndex(1)];
        ids.sort();
        assert_eq!(
            ids,
            vec![DictionaryIndex(0), DictionaryIndex(1), DictionaryIndex(3)]
        );
    }
}
