use agdb_serialize::Serialize;
use agdb_storage::StorageIndex;
use std::mem::size_of;

pub(super) struct GraphDataStorageIndexes {
    pub(super) from: StorageIndex,
    pub(super) to: StorageIndex,
    pub(super) from_meta: StorageIndex,
    pub(super) to_meta: StorageIndex,
}

impl Serialize for GraphDataStorageIndexes {
    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization() {
        let element = GraphDataStorageIndexes {
            from: StorageIndex::from(1_i64),
            to: StorageIndex::from(2_i64),
            from_meta: StorageIndex::from(-3_i64),
            to_meta: StorageIndex::from(-4_i64),
        };

        let bytes = element.serialize();
        let other = GraphDataStorageIndexes::deserialize(&bytes).unwrap();

        assert_eq!(element.from, other.from);
        assert_eq!(element.to, other.to);
        assert_eq!(element.from_meta, other.from_meta);
        assert_eq!(element.to_meta, other.to_meta);
    }
}
