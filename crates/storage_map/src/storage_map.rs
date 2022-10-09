use crate::map_data_storage::MapDataStorage;
use crate::map_impl::MapImpl;
use agdb_serialize::Serialize;
use agdb_storage::Storage;
use agdb_storage::StorageFile;
use agdb_storage::StorageIndex;
use agdb_utilities::StableHash;
use std::hash::Hash;

pub type StorageMap<K, T, Data = StorageFile> = MapImpl<K, T, MapDataStorage<K, T, Data>>;

impl<K, T, Data> StorageMap<K, T, Data>
where
    K: Clone + Default + Eq + Hash + PartialEq + StableHash + Serialize,
    T: Clone + Default + Serialize,
    Data: Storage,
{
    pub fn storage_index(&self) -> StorageIndex {
        self.map_common.data.storage_index()
    }
}
