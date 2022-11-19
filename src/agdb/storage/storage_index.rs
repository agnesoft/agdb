use crate::db::db_error::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeFixedSized;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct StorageIndex {
    pub(crate) index: u64,
}

impl StorageIndex {
    pub fn as_usize(&self) -> usize {
        self.value() as usize
    }

    pub fn is_valid(&self) -> bool {
        0 < self.index
    }

    pub fn value(&self) -> u64 {
        self.index
    }
}

impl From<u64> for StorageIndex {
    fn from(index: u64) -> Self {
        Self { index }
    }
}

impl From<usize> for StorageIndex {
    fn from(index: usize) -> Self {
        Self {
            index: index as u64,
        }
    }
}

impl Serialize for StorageIndex {
    fn serialize(&self) -> Vec<u8> {
        self.index.serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            index: u64::deserialize(bytes)?,
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
    fn is_valid() {
        assert!(!StorageIndex::default().is_valid());
        assert!(!StorageIndex::from(0_u64).is_valid());

        assert!(StorageIndex::from(1_u64).is_valid());
        assert!(StorageIndex::from(100_u64).is_valid());
    }

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
