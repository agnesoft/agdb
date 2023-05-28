use super::db_error::DbError;
use super::db_value_index::DbValueIndex;
use crate::storage::storage_value::StorageValue;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::utilities::stable_hash::StableHash;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct DbKeyValueIndex {
    pub(crate) key: DbValueIndex,
    pub(crate) value: DbValueIndex,
}

impl StableHash for DbKeyValueIndex {
    fn stable_hash(&self) -> u64 {
        [self.key.value, self.value.value].concat().stable_hash()
    }
}

impl Serialize for DbKeyValueIndex {
    fn serialize(&self) -> Vec<u8> {
        [self.key.value, self.value.value].concat()
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, DbError> {
        Ok(Self {
            key: DbValueIndex::deserialize(bytes)?,
            value: DbValueIndex::deserialize(
                &bytes[DbValueIndex::serialized_size_static() as usize..],
            )?,
        })
    }

    fn serialized_size(&self) -> u64 {
        DbValueIndex::serialized_size_static() * 2
    }
}

impl StorageValue for DbKeyValueIndex {
    fn storage_len() -> u64 {
        DbValueIndex::serialized_size_static() * 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;

    #[test]
    fn derived_from() {
        let index = DbKeyValueIndex::default();
        assert_eq!(index, index.clone());

        format!("{:?}", index);

        let mut hasher = DefaultHasher::new();
        DbKeyValueIndex::default().hash(&mut hasher);

        index.stable_hash();
    }

    #[test]
    fn serialized_size() {
        assert_eq!(DbKeyValueIndex::default().serialized_size(), 32);
    }
}
