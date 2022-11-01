use crate::db_error::DbError;
use crate::utilities::serialize::Serialize;

#[derive(Clone, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct StorageIndex {
    pub(crate) index: i64,
}

impl StorageIndex {
    pub fn as_u64(&self) -> u64 {
        self.value() as u64
    }

    pub fn as_usize(&self) -> usize {
        self.value() as usize
    }

    pub fn is_valid(&self) -> bool {
        0 < self.index
    }

    pub fn value(&self) -> i64 {
        self.index
    }
}

impl From<i64> for StorageIndex {
    fn from(index: i64) -> Self {
        Self { index }
    }
}

impl From<u64> for StorageIndex {
    fn from(index: u64) -> Self {
        Self {
            index: index as i64,
        }
    }
}

impl From<usize> for StorageIndex {
    fn from(index: usize) -> Self {
        Self {
            index: index as i64,
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_valid() {
        assert!(!StorageIndex::default().is_valid());
        assert!(!StorageIndex::from(-1_i64).is_valid());

        assert!(StorageIndex::from(1_i64).is_valid());
        assert!(StorageIndex::from(100_i64).is_valid());
    }

    #[test]
    fn ordering() {
        let mut indexes = vec![
            StorageIndex::default(),
            StorageIndex::from(100_i64),
            StorageIndex::from(-1_i64),
            StorageIndex::from(1_i64),
        ];

        indexes.sort();

        assert_eq!(
            indexes,
            vec![
                StorageIndex::from(-1_i64),
                StorageIndex::default(),
                StorageIndex::from(1_i64),
                StorageIndex::from(100_i64),
            ]
        )
    }

    #[test]
    fn serialize() {
        let index = StorageIndex::from(1_i64);
        let bytes = index.serialize();
        let other = StorageIndex::deserialize(&bytes).unwrap();

        assert_eq!(index, other);
    }
}
