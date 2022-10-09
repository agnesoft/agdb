use agdb_storage::StorageIndex;

pub(crate) struct GraphDataStorageIndexes {
    pub(crate) from: StorageIndex,
    pub(crate) to: StorageIndex,
    pub(crate) from_meta: StorageIndex,
    pub(crate) to_meta: StorageIndex,
}
