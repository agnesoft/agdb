use crate::storage_index::StorageIndex;

impl From<i64> for StorageIndex {
    fn from(index: i64) -> Self {
        Self { index }
    }
}

impl From<u64> for StorageIndex {
    fn from(index: u64) -> Self {
        Self {
            index: index as i64,
        }
    }
}

impl From<usize> for StorageIndex {
    fn from(index: usize) -> Self {
        Self {
            index: index as i64,
        }
    }
}
