use crate::DbKeyValue;
use crate::DbUserValue;
use crate::DbValue;

/// Helper type distinguishing uniform (`Single`) values
/// and multiple (`Multi`) values in database queries.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum QueryValues {
    /// Single list of properties (key-value pairs)
    /// to be applied to all elements in a query.
    Single(Vec<DbKeyValue>),

    /// List of lists of properties (key-value pairs)
    /// to be applied to all elements in a query. There
    /// must be as many lists of properties as ids
    /// in a query.
    Multi(Vec<Vec<DbKeyValue>>),
}

/// Convenient wrapper for the [`QueryBuilder`] to
/// allow properties conversions. Represents `QueryValues::Single`.
pub struct SingleValues(pub Vec<DbKeyValue>);

/// Convenient wrapper for the [`QueryBuilder`] to
/// allow properties conversions. Represents `QueryValues::Multi`.
pub struct MultiValues(pub Vec<Vec<DbKeyValue>>);

/// Convenient wrapper for the [`QueryBuilder`] to
/// allow properties conversions. Represents list
/// of property keys.
pub struct QueryKeys(pub Vec<DbValue>);

impl From<Vec<DbKeyValue>> for SingleValues {
    fn from(values: Vec<DbKeyValue>) -> Self {
        SingleValues(values)
    }
}

impl From<Vec<Vec<DbKeyValue>>> for MultiValues {
    fn from(values: Vec<Vec<DbKeyValue>>) -> Self {
        MultiValues(values)
    }
}

impl From<Vec<DbValue>> for QueryKeys {
    fn from(value: Vec<DbValue>) -> Self {
        QueryKeys(value)
    }
}

impl<T: DbUserValue> From<&T> for MultiValues {
    fn from(value: &T) -> Self {
        MultiValues(vec![value.to_db_values()])
    }
}

impl<T: DbUserValue> From<&T> for SingleValues {
    fn from(value: &T) -> Self {
        SingleValues(value.to_db_values())
    }
}

impl<T: DbUserValue> From<&Vec<T>> for MultiValues {
    fn from(value: &Vec<T>) -> Self {
        MultiValues(value.iter().map(|v| v.to_db_values()).collect())
    }
}
