use crate::StorageIndex;

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct StorageRecord {
    pub index: StorageIndex,
    pub position: u64,
    pub size: u64,
}
