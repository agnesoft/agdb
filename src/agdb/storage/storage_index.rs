use crate::db::db_error::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeFixedSized;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct StorageIndex {
    pub value: u64,
}

impl From<u64> for StorageIndex {
    fn from(index: u64) -> Self {
        Self { value: index }
    }
}

impl From<usize> for StorageIndex {
    fn from(index: usize) -> Self {
        Self {
            value: index as u64,
        }
    }
}

impl Serialize for StorageIndex {
    fn serialize(&self) -> Vec<u8> {
        self.value.serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            value: u64::deserialize(bytes)?,
        })
    }
}

impl SerializeFixedSized for StorageIndex {
    fn serialized_size() -> u64 {
        i64::serialized_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordering() {
        let mut indexes = vec![
            StorageIndex::default(),
            StorageIndex::from(100_u64),
            StorageIndex::from(u64::MAX),
            StorageIndex::from(1_u64),
        ];

        indexes.sort();

        assert_eq!(
            indexes,
            vec![
                StorageIndex::default(),
                StorageIndex::from(1_u64),
                StorageIndex::from(100_u64),
                StorageIndex::from(u64::MAX),
            ]
        )
    }

    #[test]
    fn serialize() {
        let index = StorageIndex::from(1_u64);
        let bytes = index.serialize();
        let other = StorageIndex::deserialize(&bytes).unwrap();

        assert_eq!(index, other);
    }
}
