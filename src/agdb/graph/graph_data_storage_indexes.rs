use crate::db::db_error::DbError;
use crate::storage::storage_index::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize_static::SerializeStatic;
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
            to: StorageIndex::deserialize(
                &bytes[(StorageIndex::static_serialized_size() as usize)..],
            )?,
            from_meta: StorageIndex::deserialize(
                &bytes[(StorageIndex::static_serialized_size() as usize * 2)..],
            )?,
            to_meta: StorageIndex::deserialize(
                &bytes[(StorageIndex::static_serialized_size() as usize * 3)..],
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

    fn serialized_size(&self) -> u64 {
        Self::static_serialized_size()
    }
}

impl SerializeStatic for GraphDataStorageIndexes {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let indexes = GraphDataStorageIndexes {
            from: StorageIndex::default(),
            to: StorageIndex::default(),
            from_meta: StorageIndex::default(),
            to_meta: StorageIndex::default(),
        };

        assert_ne!(indexes.serialized_size(), 0);
        assert_eq!(
            indexes.serialized_size(),
            GraphDataStorageIndexes::static_serialized_size()
        );
    }
}
