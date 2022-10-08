use agdb_serialize::Serialize;

pub(super) struct DictionaryDataStorageIndexes {
    pub(super) index: i64,
    pub(super) values: i64,
}

impl Serialize for DictionaryDataStorageIndexes {
    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        Ok(DictionaryDataStorageIndexes {
            index: i64::deserialize(bytes)?,
            values: i64::deserialize(&bytes[std::mem::size_of::<i64>()..])?,
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
            index: 1,
            values: 2,
        };

        let bytes = element.serialize();
        let other = DictionaryDataStorageIndexes::deserialize(&bytes).unwrap();

        assert_eq!(element.index, other.index);
        assert_eq!(element.values, other.values);
    }
}
