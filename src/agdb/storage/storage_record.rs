use crate::db::db_error::DbError;
use crate::storage::storage_index::StorageIndex;
use crate::utilities::serialize::Serialize;

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct StorageRecord {
    pub index: StorageIndex,
    pub position: u64,
    pub size: u64,
}

impl Serialize for StorageRecord {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(StorageRecord {
            index: StorageIndex::deserialize(bytes)?,
            position: 0,
            size: u64::deserialize(&bytes[(StorageIndex::fixed_size() as usize)..])?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = self.index.serialize();
        bytes.extend(self.size.serialize());

        bytes
    }

    fn fixed_size() -> u64 {
        StorageIndex::fixed_size() + u64::fixed_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn derived_from_debug() {
        let record = StorageRecord::default();
        format!("{:?}", record);
    }

    #[test]
    fn derived_from_ord() {
        let record = StorageRecord::default();
        assert_eq!(record.cmp(&record), Ordering::Equal);
    }

    #[test]
    fn serialize() {
        let bytes = StorageRecord {
            index: StorageIndex::from(1_i64),
            position: 64,
            size: 128,
        }
        .serialize();
        let record = StorageRecord::deserialize(&bytes).unwrap();

        assert_eq!(record.index, StorageIndex::from(1_i64));
        assert_eq!(record.position, 0);
        assert_eq!(record.size, 128);
    }
}
