use crate::db::db_error::DbError;
use crate::old_storage::storage_index::StorageIndex;
use crate::utilities::old_serialize::OldSerialize;
use std::mem::size_of;

pub(crate) struct GraphDataStorageIndexes {
    pub(crate) from: StorageIndex,
    pub(crate) to: StorageIndex,
    pub(crate) from_meta: StorageIndex,
    pub(crate) to_meta: StorageIndex,
}

impl OldSerialize for GraphDataStorageIndexes {
    fn old_deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(GraphDataStorageIndexes {
            from: StorageIndex::old_deserialize(bytes)?,
            to: StorageIndex::old_deserialize(&bytes[(StorageIndex::fixed_size() as usize)..])?,
            from_meta: StorageIndex::old_deserialize(
                &bytes[(StorageIndex::fixed_size() as usize * 2)..],
            )?,
            to_meta: StorageIndex::old_deserialize(
                &bytes[(StorageIndex::fixed_size() as usize * 3)..],
            )?,
        })
    }

    fn old_serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        bytes.reserve(4 * size_of::<i64>());

        bytes.extend(self.from.old_serialize());
        bytes.extend(self.to.old_serialize());
        bytes.extend(self.from_meta.old_serialize());
        bytes.extend(self.to_meta.old_serialize());

        bytes
    }
}
