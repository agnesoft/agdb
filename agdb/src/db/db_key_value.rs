use crate::collections::vec::VecValue;
use crate::db::db_value_index::DbValueIndex;
use crate::storage::Storage;
use crate::storage::StorageIndex;
use crate::utilities::serialize::Serialize;
use crate::utilities::serialize::SerializeStatic;
use crate::DbError;
use crate::DbValue;
use crate::StorageData;

/// Database key-value pair (aka property) attached to
/// database elements. It can be constructed from a
/// tuple of types that are convertible to `DbValue`.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::AgdbDeSerialize))]
#[cfg_attr(feature = "api", agdb::api_def())]
pub struct DbKeyValue {
    /// Key of the property
    pub key: DbValue,

    /// Value of the property
    pub value: DbValue,
}

impl<K, T> From<(K, T)> for DbKeyValue
where
    K: Into<DbValue>,
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
    fn store<D: StorageData>(&self, storage: &mut Storage<D>) -> Result<Vec<u8>, DbError> {
        let key_index = self.key.store_db_value(storage)?;
        let value_index = self.value.store_db_value(storage)?;
        Ok([key_index.data(), value_index.data()].concat())
    }

    fn load<D: StorageData>(storage: &Storage<D>, bytes: &[u8]) -> Result<Self, DbError> {
        let key_index = DbValueIndex::deserialize(bytes)?;
        let value_index =
            DbValueIndex::deserialize(&bytes[key_index.serialized_size() as usize..])?;
        let key = DbValue::load_db_value(key_index, storage)?;
        let value = DbValue::load_db_value(value_index, storage)?;
        Ok(Self { key, value })
    }

    fn remove<D: StorageData>(storage: &mut Storage<D>, bytes: &[u8]) -> Result<(), DbError> {
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
        let _ = format!(
            "{:?}",
            DbKeyValue {
                key: DbValue::I64(0),
                value: DbValue::I64(0)
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            DbKeyValue {
                key: DbValue::I64(0),
                value: DbValue::I64(0)
            },
            DbKeyValue {
                key: DbValue::I64(0),
                value: DbValue::I64(0)
            }
        );
    }

    #[test]
    fn derived_from_partial_ord() {
        let element = DbKeyValue {
            key: DbValue::I64(0),
            value: DbValue::I64(0),
        };
        let other = DbKeyValue {
            key: DbValue::I64(0),
            value: DbValue::I64(0),
        };
        assert!(element <= other);
    }

    #[test]
    fn derived_from_ord() {
        let element = DbKeyValue {
            key: DbValue::I64(0),
            value: DbValue::I64(0),
        };
        assert_eq!(element.cmp(&element), std::cmp::Ordering::Equal);
    }
}
