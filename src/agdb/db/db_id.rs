use super::db_error::DbError;
use crate::collections::vec::VecValue;
use crate::utilities::serialize::{Serialize, SerializeStatic};
use crate::utilities::stable_hash::StableHash;

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DbId(pub i64);

impl StableHash for DbId {
    fn stable_hash(&self) -> u64 {
        self.0.stable_hash()
    }
}

impl Serialize for DbId {
    fn serialize(&self) -> Vec<u8> {
        self.0.serialize()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self(i64::deserialize(bytes)?))
    }

    fn serialized_size(&self) -> u64 {
        self.0.serialized_size()
    }
}

impl SerializeStatic for DbId {}

impl VecValue for DbId {
    fn storage_len() -> u64 {
        Self::serialized_size_static()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize() {
        let id = DbId(1);
        assert_eq!(id.serialized_size(), 8);
        let bytes = id.serialize();
        let other = DbId::deserialize(&bytes).unwrap();
        assert_eq!(id, other);
    }
}
