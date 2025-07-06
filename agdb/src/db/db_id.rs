use crate::DbError;
use crate::StorageData;
use crate::collections::vec::VecValue;
use crate::storage::Storage;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::stable_hash::StableHash;

/// Database id is a wrapper around `i64`.
/// The id is an identifier of a database element
/// both nodes and edges. The positive ids represent nodes,
/// negative ids represent edges. The value of `0` is
/// logically invalid (there cannot be element with id 0) and a default.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::AgdbDeSerialize))]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct DbId(pub i64);

impl StableHash for DbId {
    fn stable_hash(&self) -> u64 {
        self.0.stable_hash()
    }
}

impl<D: StorageData> VecValue<D> for DbId {
    fn store(&self, _storage: &mut Storage<D>) -> Result<Vec<u8>, DbError> {
        Ok(self.0.serialize())
    }

    fn load(_storage: &Storage<D>, bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self(i64::deserialize(bytes)?))
    }

    fn remove(_storage: &mut Storage<D>, _bytes: &[u8]) -> Result<(), DbError> {
        Ok(())
    }

    fn storage_len() -> u64 {
        i64::serialized_size_static()
    }
}

impl DbId {
    pub fn as_index(&self) -> u64 {
        if self.0 < 0 {
            (-self.0) as u64
        } else {
            self.0 as u64
        }
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
