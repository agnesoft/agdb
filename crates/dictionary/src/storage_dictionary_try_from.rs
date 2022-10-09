use super::dictionary_data_storage::DictionaryDataStorage;
use super::dictionary_data_storage_indexes::DictionaryDataStorageIndexes;
use super::dictionary_value::DictionaryValue;
use crate::StorageDictionary;
use agdb_db_error::DbError;
use agdb_multi_map::StorageMultiMap;
use agdb_serialize::Serialize;
use agdb_storage::Storage;
use agdb_storage::StorageIndex;
use agdb_storage_vec::StorageVec;
use agdb_utilities::StableHash;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

impl<T, Data> TryFrom<Rc<RefCell<Data>>> for StorageDictionary<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(storage: Rc<RefCell<Data>>) -> Result<Self, Self::Error> {
        let index = StorageMultiMap::<u64, i64, Data>::try_from(storage.clone())?;
        let mut values = StorageVec::<DictionaryValue<T>, Data>::try_from(storage.clone())?;
        values.push(&DictionaryValue::default())?;

        let storage_index = storage.borrow_mut().insert(&DictionaryDataStorageIndexes {
            index: index.storage_index(),
            values: values.storage_index(),
        })?;

        Ok(StorageDictionary::<T, Data> {
            data: DictionaryDataStorage::<T, Data> {
                storage,
                storage_index,
                index,
                values,
            },
            phantom_data: PhantomData,
        })
    }
}

impl<T, Data> TryFrom<(Rc<RefCell<Data>>, StorageIndex)> for StorageDictionary<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (Rc<RefCell<Data>>, StorageIndex),
    ) -> Result<Self, Self::Error> {
        let indexes = storage_with_index
            .0
            .borrow_mut()
            .value::<DictionaryDataStorageIndexes>(&storage_with_index.1)?;
        let index = StorageMultiMap::<u64, i64, Data>::try_from((
            storage_with_index.0.clone(),
            indexes.index,
        ))?;
        let values = StorageVec::<DictionaryValue<T>, Data>::try_from((
            storage_with_index.0.clone(),
            indexes.values,
        ))?;

        Ok(StorageDictionary::<T, Data> {
            data: DictionaryDataStorage::<T, Data> {
                storage: storage_with_index.0,
                storage_index: storage_with_index.1,
                index,
                values,
            },
            phantom_data: PhantomData,
        })
    }
}
