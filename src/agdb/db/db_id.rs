use super::db_error::DbError;
use crate::{
    storage::{storage_value::StorageValue, Storage},
    utilities::{serialize::Serialize, stable_hash::StableHash},
};

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

impl StorageValue for DbId {
    fn store<S: Storage>(&self, storage: &mut S) -> Result<Vec<u8>, DbError> {
        self.0.store(storage)
    }

    fn load<S: Storage>(storage: &S, bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self(i64::load(storage, bytes)?))
    }

    fn remove<S: Storage>(storage: &mut S, bytes: &[u8]) -> Result<(), DbError> {
        i64::remove(storage, bytes)
    }

    fn storage_len() -> u64 {
        i64::storage_len()
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
}
