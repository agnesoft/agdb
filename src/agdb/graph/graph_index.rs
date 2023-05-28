use crate::collections::vec::VecValue;
use crate::db::db_error::DbError;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::stable_hash::StableHash;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, Hash, PartialEq, PartialOrd)]
pub struct GraphIndex {
    pub index: i64,
}

impl GraphIndex {
    pub fn is_edge(&self) -> bool {
        self.index < 0
    }

    pub fn is_node(&self) -> bool {
        0 < self.index
    }

    pub fn is_valid(&self) -> bool {
        self.index != 0
    }

    pub(crate) fn as_u64(&self) -> u64 {
        if self.is_edge() {
            (-self.index) as u64
        } else {
            self.index as u64
        }
    }

    pub(crate) fn as_usize(&self) -> usize {
        if self.is_edge() {
            (-self.index) as usize
        } else {
            self.index as usize
        }
    }
}

impl From<i64> for GraphIndex {
    fn from(index: i64) -> Self {
        Self { index }
    }
}

impl StableHash for GraphIndex {
    fn stable_hash(&self) -> u64 {
        self.index.stable_hash()
    }
}

impl Serialize for GraphIndex {
    fn serialize(&self) -> Vec<u8> {
        self.index.serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            index: i64::deserialize(bytes)?,
        })
    }

    fn serialized_size(&self) -> u64 {
        self.index.serialized_size()
    }
}

impl SerializeStatic for GraphIndex {}

impl VecValue for GraphIndex {
    fn storage_len() -> u64 {
        Self::serialized_size_static()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_edge() {
        assert!(!GraphIndex::from(1).is_edge());
        assert!(!GraphIndex::default().is_edge());
        assert!(GraphIndex::from(-1).is_edge());
    }

    #[test]
    fn is_node() {
        assert!(GraphIndex::from(1).is_node());
        assert!(!GraphIndex::default().is_node());
        assert!(!GraphIndex::from(-1).is_node());
    }

    #[test]
    fn serialize() {
        let index = GraphIndex { index: 1 };
        assert_eq!(index.serialized_size(), 8);
        let bytes = index.serialize();
        let other = GraphIndex::deserialize(&bytes).unwrap();
        assert_eq!(index, other);
    }
}
