use crate::map_data_storage::MapDataStorage;
use crate::StorageMap;
use agdb_db_error::DbError;
use agdb_map_common::MapCommon;
use agdb_map_common::MapValue;
use agdb_serialize::Serialize;
use agdb_storage::Storage;
use agdb_storage::StorageIndex;
use agdb_utilities::StableHash;
use std::cell::RefCell;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::size_of;
use std::rc::Rc;

impl<K, T, Data> TryFrom<Rc<RefCell<Data>>> for StorageMap<K, T, Data>
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
            map_common: MapCommon::from(MapDataStorage::try_from(storage)?),
            phantom_data: PhantomData,
        })
    }
}

impl<K, T, Data> TryFrom<(Rc<RefCell<Data>>, StorageIndex)> for StorageMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (Rc<RefCell<Data>>, StorageIndex),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            map_common: MapCommon::from(MapDataStorage::try_from(storage_with_index)?),
            phantom_data: PhantomData,
        })
    }
}
