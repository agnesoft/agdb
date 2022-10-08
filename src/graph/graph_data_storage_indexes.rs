use agdb_serialize::Serialize;

pub(super) struct GraphDataStorageIndexes {
    pub(super) from: i64,
    pub(super) to: i64,
    pub(super) from_meta: i64,
    pub(super) to_meta: i64,
}

impl Serialize for GraphDataStorageIndexes {
    fn deserialize(bytes: &[u8]) -> Result<Self, crate::DbError> {
        Ok(GraphDataStorageIndexes {
            from: i64::deserialize(bytes)?,
            to: i64::deserialize(&bytes[std::mem::size_of::<i64>()..])?,
            from_meta: i64::deserialize(&bytes[(std::mem::size_of::<i64>() * 2)..])?,
            to_meta: i64::deserialize(&bytes[(std::mem::size_of::<i64>() * 3)..])?,
        })
    }

    fn serialize(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        bytes.reserve(4 * std::mem::size_of::<i64>());

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
            from: 1,
            to: 2,
            from_meta: -3,
            to_meta: -4,
        };

        let bytes = element.serialize();
        let other = GraphDataStorageIndexes::deserialize(&bytes).unwrap();

        assert_eq!(element.from, other.from);
        assert_eq!(element.to, other.to);
        assert_eq!(element.from_meta, other.from_meta);
        assert_eq!(element.to_meta, other.to_meta);
    }
}
