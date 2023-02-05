use crate::DbKey;
use crate::DbValue;

#[derive(Debug, Clone, PartialEq)]
pub struct DbKeyValue {
    pub key: DbKey,
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
