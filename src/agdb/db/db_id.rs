use super::db_error::DbError;
use crate::collections::vec::VecValue;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
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
    use std::cmp::Ordering;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;

    #[test]
    fn derived_from_hash() {
        let mut hasher = DefaultHasher::new();
        DbId(1).hash(&mut hasher);
        assert_ne!(hasher.finish(), 0);
    }

    #[test]
    fn derived_from_ord() {
        assert_eq!(DbId(1).cmp(&DbId(1)), Ordering::Equal);
    }

    #[test]
    fn derived_from_partial_ord() {
        let mut ids = vec![DbId(3), DbId(0), DbId(-1)];
        ids.sort();
        assert_eq!(ids, vec![DbId(-1), DbId(0), DbId(3)]);
    }

    #[test]
    fn serialize() {
        let id = DbId(1);
        assert_eq!(id.serialized_size(), 8);
        let bytes = id.serialize();
        let other = DbId::deserialize(&bytes).unwrap();
        assert_eq!(id, other);
    }
}
