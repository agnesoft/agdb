use super::dictionary_data::DictionaryData;
use super::dictionary_value::DictionaryValue;
use crate::storage::FileStorageData;
use crate::storage::StableHash;
use crate::storage::Storage;
use crate::storage::StorageData;
use crate::storage::StorageHashMultiMap;
use crate::storage::StorageVec;
use crate::DbError;
use serialize::Serialize;

pub(crate) struct DictionaryDataStorage<T, Data = FileStorageData>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: StorageData,
{
    pub(super) storage: std::rc::Rc<std::cell::RefCell<Storage<Data>>>,
    pub(super) storage_index: i64,
    pub(super) index: StorageHashMultiMap<u64, i64, Data>,
    pub(super) values: StorageVec<DictionaryValue<T>, Data>,
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
        let values_index = self.values.storage_index();
        self.storage.borrow_mut().value_at::<u64>(
            values_index,
            StorageVec::<DictionaryValue<T>>::value_offset(index as u64) + i64::serialized_size(),
        )
    }

    fn meta(&self, index: i64) -> Result<i64, DbError> {
        let values_index = self.values.storage_index();
        self.storage.borrow_mut().value_at::<i64>(
            values_index,
            StorageVec::<DictionaryValue<T>>::value_offset(index as u64),
        )
    }

    fn remove(&mut self, hash: u64, index: i64) -> Result<(), DbError> {
        self.index.remove_value(&hash, &index)
    }

    fn set_hash(&mut self, index: i64, hash: u64) -> Result<(), DbError> {
        let values_index = self.values.storage_index();
        self.storage.borrow_mut().insert_at(
            values_index,
            StorageVec::<DictionaryValue<T>>::value_offset(index as u64) + u64::serialized_size(),
            &hash,
        )
    }

    fn set_meta(&mut self, index: i64, meta: i64) -> Result<(), DbError> {
        let values_index = self.values.storage_index();
        self.storage.borrow_mut().insert_at(
            values_index,
            StorageVec::<DictionaryValue<T>>::value_offset(index as u64),
            &meta,
        )
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
