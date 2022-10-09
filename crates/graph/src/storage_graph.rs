use super::graph_data_storage::GraphDataStorage;
use super::graph_impl::GraphImpl;
use agdb_storage::Storage;
use agdb_storage::StorageFile;
use agdb_storage::StorageIndex;

pub type StorageGraph<Data = StorageFile> = GraphImpl<GraphDataStorage<Data>>;

impl<Data> StorageGraph<Data>
where
    Data: Storage,
{
    pub fn storage_index(&self) -> StorageIndex {
        self.data.storage_index.clone()
    }
}
