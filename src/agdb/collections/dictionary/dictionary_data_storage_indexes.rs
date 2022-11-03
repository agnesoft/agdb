use crate::db::db_error::DbError;
use crate::storage::storage_index::StorageIndex;
use crate::utilities::serialize::Serialize;
use std::mem::size_of;

pub(crate) struct DictionaryDataStorageIndexes {
    pub(crate) index: StorageIndex,
    pub(crate) values: StorageIndex,
}

impl Serialize for DictionaryDataStorageIndexes {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(DictionaryDataStorageIndexes {
            index: StorageIndex::deserialize(bytes)?,
            values: StorageIndex::deserialize(
                &bytes[(StorageIndex::serialized_size() as usize)..],
            )?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        bytes.reserve(4 * size_of::<i64>());

        bytes.extend(self.index.serialize());
        bytes.extend(self.values.serialize());

        bytes
    }
}
