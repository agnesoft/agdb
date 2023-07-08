use super::db_error::DbError;
use super::db_value_index::DbValueIndex;
use crate::collections::vec::VecValue;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::DbKey;
use crate::DbValue;

/// Database key-value pair (aka property) attached to
/// database elements. It can be constructed from a
/// tuple of types that are convertible to `DbValue`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DbKeyValue {
    /// Key of the property
    pub key: DbKey,

    /// Value of the property
    pub value: DbValue,
}

impl<K, T> From<(K, T)> for DbKeyValue
where
    K: Into<DbKey>,
    T: Into<DbValue>,
{
    fn from(value: (K, T)) -> Self {
        DbKeyValue {
            key: value.0.into(),
            value: value.1.into(),
        }
    }
}

impl VecValue for DbKeyValue {
    fn store<S: Storage>(&self, storage: &mut S) -> Result<Vec<u8>, DbError> {
        let key_index = self.key.store_db_value(storage)?;
        let value_index = self.value.store_db_value(storage)?;
        Ok([key_index.value, value_index.value].concat())
    }

    fn load<S: Storage>(storage: &S, bytes: &[u8]) -> Result<Self, DbError> {
        let key_index = DbValueIndex::deserialize(bytes)?;
        let value_index =
            DbValueIndex::deserialize(&bytes[key_index.serialized_size() as usize..])?;
        let key = DbValue::load_db_value(key_index, storage)?;
        let value = DbValue::load_db_value(value_index, storage)?;
        Ok(Self { key, value })
    }

    fn remove<S: Storage>(storage: &mut S, bytes: &[u8]) -> Result<(), DbError> {
        let key_index = DbValueIndex::deserialize(bytes)?;
        let value_index =
            DbValueIndex::deserialize(&bytes[key_index.serialized_size() as usize..])?;

        if !key_index.is_value() {
            storage.remove(StorageIndex(key_index.index()))?;
        }

        if !value_index.is_value() {
            storage.remove(StorageIndex(value_index.index()))?;
        }

        Ok(())
    }

    fn storage_len() -> u64 {
        DbValueIndex::serialized_size_static() * 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            DbKeyValue {
                key: DbKey::Int(0),
                value: DbKey::Int(0)
            }
        );
    }
    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            DbKeyValue {
                key: DbKey::Int(0),
                value: DbKey::Int(0)
            },
            DbKeyValue {
                key: DbKey::Int(0),
                value: DbKey::Int(0)
            }
        );
    }
}
