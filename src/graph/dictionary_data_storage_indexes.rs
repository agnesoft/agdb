use agdb_serialize::Serialize;
use agdb_storage::StorageIndex;

pub(super) struct DictionaryDataStorageIndexes {
    pub(super) index: StorageIndex,
    pub(super) values: StorageIndex,
}

impl Serialize for DictionaryDataStorageIndexes {
    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        Ok(DictionaryDataStorageIndexes {
            index: StorageIndex::deserialize(bytes)?,
            values: StorageIndex::deserialize(
                &bytes[(StorageIndex::serialized_size() as usize)..],
            )?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        bytes.reserve(4 * std::mem::size_of::<i64>());

        bytes.extend(self.index.serialize());
        bytes.extend(self.values.serialize());

        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization() {
        let element = DictionaryDataStorageIndexes {
            index: StorageIndex::from(1_i64),
            values: StorageIndex::from(2_i64),
        };

        let bytes = element.serialize();
        let other = DictionaryDataStorageIndexes::deserialize(&bytes).unwrap();

        assert_eq!(element.index, other.index);
        assert_eq!(element.values, other.values);
    }
}
