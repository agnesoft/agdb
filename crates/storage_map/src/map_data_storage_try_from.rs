use crate::map_data_storage::MapDataStorage;
use agdb_db_error::DbError;
use agdb_map_common::MapValue;
use agdb_utilities::StableHash;
use agdb_serialize::Serialize;
use agdb_storage::Storage;
use agdb_storage::StorageIndex;
use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::size_of;
use std::rc::Rc;

impl<K, T, Data> TryFrom<Rc<RefCell<Data>>> for MapDataStorage<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(storage: Rc<RefCell<Data>>) -> Result<Self, Self::Error> {
        let storage_index = storage.borrow_mut().insert(&0_u64)?;
        storage.borrow_mut().insert_at(
            &storage_index,
            size_of::<u64>() as u64,
            &vec![MapValue::<K, T>::default()],
        )?;

        Ok(Self {
            storage,
            storage_index,
            count: 0,
            capacity: 1,
            phantom_data: PhantomData,
        })
    }
}

impl<K, T, Data> TryFrom<(Rc<RefCell<Data>>, StorageIndex)> for MapDataStorage<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (Rc<RefCell<Data>>, StorageIndex),
    ) -> Result<Self, Self::Error> {
        let count = storage_with_index
            .0
            .borrow_mut()
            .value_at::<u64>(&storage_with_index.1, 0)?;
        let capacity = storage_with_index
            .0
            .borrow_mut()
            .value_at::<u64>(&storage_with_index.1, size_of::<u64>() as u64)?;

        Ok(Self {
            storage: storage_with_index.0,
            storage_index: storage_with_index.1,
            count,
            capacity,
            phantom_data: PhantomData,
        })
    }
}
