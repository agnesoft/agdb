use crate::storage_record::StorageRecord;
use crate::StorageIndex;
use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use std::mem::size_of;

impl Serialize for StorageRecord {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(StorageRecord {
            index: StorageIndex::deserialize(bytes)?,
            position: 0,
            size: u64::deserialize(&bytes[(StorageIndex::serialized_size() as usize)..])?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes = self.index.serialize();
        bytes.extend(self.size.serialize());

        bytes
    }

    fn serialized_size() -> u64 {
        StorageIndex::serialized_size() + size_of::<u64>() as u64
    }
}
