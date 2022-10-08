use crate::storage_index::StorageIndex;
use agdb_db_error::DbError;
use agdb_serialize::Serialize;

impl Serialize for StorageIndex {
    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            index: i64::deserialize(bytes)?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        self.index.serialize()
    }
}
