use agdb_serialize::Serialize;
use agdb_storage::StorageIndex;

pub(super) struct GraphDataStorageIndexes {
    pub(super) from: StorageIndex,
    pub(super) to: StorageIndex,
    pub(super) from_meta: StorageIndex,
    pub(super) to_meta: StorageIndex,
}

impl Serialize for GraphDataStorageIndexes {
    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        Ok(GraphDataStorageIndexes {
            from: StorageIndex::from(i64::deserialize(bytes)?),
            to: StorageIndex::from(i64::deserialize(&bytes[std::mem::size_of::<i64>()..])?),
            from_meta: StorageIndex::from(i64::deserialize(
                &bytes[(std::mem::size_of::<i64>() * 2)..],
            )?),
            to_meta: StorageIndex::from(i64::deserialize(
                &bytes[(std::mem::size_of::<i64>() * 3)..],
            )?),
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        bytes.reserve(4 * std::mem::size_of::<i64>());

        bytes.extend(self.from.value().serialize());
        bytes.extend(self.to.value().serialize());
        bytes.extend(self.from_meta.value().serialize());
        bytes.extend(self.to_meta.value().serialize());

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
