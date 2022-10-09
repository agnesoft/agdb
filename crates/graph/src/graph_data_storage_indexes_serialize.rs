use crate::graph_data_storage_indexes::GraphDataStorageIndexes;
use agdb_db_error::DbError;
use agdb_serialize::Serialize;
use agdb_storage::StorageIndex;
use std::mem::size_of;

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
