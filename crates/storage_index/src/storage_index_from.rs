use crate::storage_index::StorageIndex;

impl From<i64> for StorageIndex {
    fn from(index: i64) -> Self {
        StorageIndex { index }
    }
}

impl From<u64> for StorageIndex {
    fn from(index: u64) -> Self {
        StorageIndex {
            index: index as i64,
        }
    }
}
