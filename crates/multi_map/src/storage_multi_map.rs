use crate::map_data_memory::MapDataMemory;
use crate::multi_map::MultiMap;
use crate::multi_map_impl::MultiMapImpl;
use agdb_db_error::DbError;
use agdb_map_common::MapCommon;
use agdb_serialize::Serialize;
use agdb_storage::Storage;
use agdb_storage::StorageFile;
use agdb_storage::StorageIndex;
use agdb_storage_map::MapDataStorage;
use agdb_utilities::StableHash;
use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

pub type StorageMultiMap<K, T, Data = StorageFile> = MultiMapImpl<K, T, MapDataStorage<K, T, Data>>;

impl<K, T, Data> StorageMultiMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
    Data: Storage,
{
    pub fn storage_index(&self) -> StorageIndex {
        self.map_common.data.storage_index()
    }

    pub fn to_multi_map(&self) -> Result<MultiMap<K, T>, DbError> {
        Ok(MultiMap::<K, T> {
            map_common: MapCommon::from(MapDataMemory::<K, T> {
                data: self.map_common.data.values()?,
                count: self.map_common.data.count(),
            }),
        })
    }
}

impl<K, T, Data> TryFrom<Rc<RefCell<Data>>> for StorageMultiMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(storage: Rc<RefCell<Data>>) -> Result<Self, Self::Error> {
        Ok(Self {
            map_common: MapCommon::from(MapDataStorage::try_from(storage)?),
        })
    }
}

impl<K, T, Data> TryFrom<(Rc<RefCell<Data>>, StorageIndex)> for StorageMultiMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Eq + PartialEq + Serialize,
    Data: Storage,
{
    type Error = DbError;

    fn try_from(
        storage_with_index: (Rc<RefCell<Data>>, StorageIndex),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            map_common: MapCommon::from(MapDataStorage::try_from(storage_with_index)?),
        })
    }
}
