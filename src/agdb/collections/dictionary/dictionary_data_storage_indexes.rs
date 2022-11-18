use crate::db::db_error::DbError;
use crate::storage::storage_index::StorageIndex;
use crate::utilities::old_serialize::OldSerialize;
use std::mem::size_of;

pub(crate) struct DictionaryDataStorageIndexes {
    pub(crate) index: StorageIndex,
    pub(crate) values: StorageIndex,
}

impl OldSerialize for DictionaryDataStorageIndexes {
    fn old_deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(DictionaryDataStorageIndexes {
            index: StorageIndex::old_deserialize(bytes)?,
            values: StorageIndex::old_deserialize(&bytes[(StorageIndex::fixed_size() as usize)..])?,
        })
    }

    fn old_serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        bytes.reserve(4 * size_of::<i64>());

        bytes.extend(self.index.old_serialize());
        bytes.extend(self.values.old_serialize());

        bytes
    }
}
