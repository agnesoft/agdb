use std::mem::size_of;

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct StorageIndex {
    pub(crate) index: i64,
}

impl StorageIndex {
    pub fn as_u64(&self) -> u64 {
        self.value() as u64
    }

    pub fn as_usize(&self) -> usize {
        self.value() as usize
    }

    pub fn is_valid(&self) -> bool {
        0 < self.index
    }

    pub fn serialized_size() -> u64 {
        size_of::<i64>() as u64
    }

    pub fn value(&self) -> i64 {
        self.index
    }
}
