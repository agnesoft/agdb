use crate::db::db_error::DbError;
use crate::storage::storage_index::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize_static::SerializeStatic;

pub struct DictionaryDataStorageIndexes {
    pub index_index: StorageIndex,
    pub counts_index: StorageIndex,
    pub hashes_index: StorageIndex,
    pub values_index: StorageIndex,
}

impl Serialize for DictionaryDataStorageIndexes {
    fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();
        bytes.reserve(self.serialized_size() as usize);
        bytes.extend(self.index_index.serialize());
        bytes.extend(self.counts_index.serialize());
        bytes.extend(self.hashes_index.serialize());
        bytes.extend(self.values_index.serialize());

        bytes
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        if bytes.len() < Self::static_serialized_size() as usize {
            return Err(DbError::from(
                "DictionaryDataStorageIndexes deserialization error: not enough data",
            ));
        }

        Ok(DictionaryDataStorageIndexes {
            index_index: StorageIndex::deserialize(
                &bytes[u64::static_serialized_size() as usize..],
            )?,
            counts_index: StorageIndex::deserialize(
                &bytes[(u64::static_serialized_size() + StorageIndex::static_serialized_size())
                    as usize..],
            )?,
            hashes_index: StorageIndex::deserialize(
                &bytes[(u64::static_serialized_size() + StorageIndex::static_serialized_size() * 2)
                    as usize..],
            )?,
            values_index: StorageIndex::deserialize(
                &bytes[(u64::static_serialized_size() + StorageIndex::static_serialized_size() * 3)
                    as usize..],
            )?,
        })
    }

    fn serialized_size(&self) -> u64 {
        Self::static_serialized_size()
    }
}

impl SerializeStatic for DictionaryDataStorageIndexes {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bad_deserialize() {
        assert_eq!(
            DictionaryDataStorageIndexes::deserialize(&Vec::<u8>::new())
                .err()
                .unwrap(),
            DbError::from("DictionaryDataStorageIndexes deserialization error: not enough data")
        );
    }
}
