use crate::db::db_error::DbError;
use crate::storage::storage_index::StorageIndex;
use crate::utilities::serialize::Serialize;
use std::mem::size_of;

pub(crate) struct GraphDataStorageIndexes {
    pub(crate) from: StorageIndex,
    pub(crate) to: StorageIndex,
    pub(crate) from_meta: StorageIndex,
    pub(crate) to_meta: StorageIndex,
}

impl Serialize for GraphDataStorageIndexes {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(GraphDataStorageIndexes {
            from: StorageIndex::deserialize(bytes)?,
            to: StorageIndex::deserialize(&bytes[(StorageIndex::serialized_size() as usize)..])?,
            from_meta: StorageIndex::deserialize(
                &bytes[(StorageIndex::serialized_size() as usize * 2)..],
            )?,
            to_meta: StorageIndex::deserialize(
                &bytes[(StorageIndex::serialized_size() as usize * 3)..],
            )?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        bytes.reserve(4 * size_of::<i64>());

        bytes.extend(self.from.serialize());
        bytes.extend(self.to.serialize());
        bytes.extend(self.from_meta.serialize());
        bytes.extend(self.to_meta.serialize());

        bytes
    }
}
