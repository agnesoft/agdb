use super::dictionary_data_storage::DictionaryDataStorage;
use super::dictionary_impl::DictionaryImpl;
use agdb_serialize::Serialize;
use agdb_storage::Storage;
use agdb_storage::StorageFile;
use agdb_storage::StorageIndex;
use agdb_utilities::StableHash;

pub type StorageDictionary<T, Data = StorageFile> =
    DictionaryImpl<T, DictionaryDataStorage<T, Data>>;

impl<T, Data> StorageDictionary<T, Data>
where
    T: Clone + Default + Eq + PartialEq + StableHash + Serialize,
    Data: Storage,
{
    pub fn storage_index(&self) -> StorageIndex {
        self.data.storage_index.clone()
    }
}
