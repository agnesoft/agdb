use super::dictionary_data::DictionaryData;
use super::dictionary_value::DictionaryValue;
use crate::storage::FileStorageData;
use crate::storage::Serialize;
use crate::storage::StableHash;
use crate::storage::Storage;
use crate::storage::StorageData;
use crate::storage::StorageHashMultiMap;
use crate::storage::StorageVec;
use crate::DbError;

pub(crate) struct DictionaryDataStorage<T, Data = FileStorageData>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: StorageData,
{
    pub(super) storage: std::rc::Rc<std::cell::RefCell<Storage<Data>>>,
    pub(super) storage_index: i64,
    pub(super) index: StorageHashMultiMap<u64, i64>,
    pub(super) values: StorageVec<DictionaryValue<T>>,
}

impl<T, Data> DictionaryData<T> for DictionaryDataStorage<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: StorageData,
{
    fn capacity(&self) -> u64 {
        self.values.len() as u64
    }

    fn commit(&mut self) -> Result<(), DbError> {
        self.storage.borrow_mut().commit()
    }

    fn indexes(&self, hash: u64) -> Result<Vec<i64>, DbError> {
        self.index.values(&hash)
    }

    fn insert(&mut self, hash: u64, index: i64) -> Result<(), DbError> {
        self.index.insert(hash, index)
    }

    fn hash(&self, index: i64) -> Result<u64, DbError> {
        Ok(self.values[index as usize].hash)
    }

    fn meta(&self, index: i64) -> Result<i64, DbError> {
        Ok(self.values[index as usize].meta)
    }

    fn remove(&mut self, hash: u64, index: i64) -> Result<(), DbError> {
        self.index.remove_value(&hash, &index)
    }

    fn set_hash(&mut self, index: i64, hash: u64) -> Result<(), DbError> {
        self.values[index as usize].hash = hash;

        Ok(())
    }

    fn set_meta(&mut self, index: i64, meta: i64) -> Result<(), DbError> {
        self.values[index as usize].meta = meta;

        Ok(())
    }

    fn set_value(&mut self, index: i64, value: DictionaryValue<T>) -> Result<(), DbError> {
        if self.capacity() == index as u64 {
            self.values.push(&value)
        } else {
            self.values.set_value(index as u64, &value)
        }
    }

    fn transaction(&mut self) {
        self.storage.borrow_mut().transaction()
    }

    fn value(&self, index: i64) -> Result<DictionaryValue<T>, crate::DbError> {
        self.values.value(index as u64)
    }
}
