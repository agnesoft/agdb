use crate::StorageIndex;
use std::mem::size_of;

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct StorageRecord {
    pub index: StorageIndex,
    pub position: u64,
    pub size: u64,
}

impl StorageRecord {
    pub fn serialized_size() -> u64 {
        StorageIndex::serialized_size() + (size_of::<u64>() as u64)
    }
}
