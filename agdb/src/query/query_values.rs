use crate::db::db_key_value::DbKeyValue;
use crate::{DbKey, DbUserValue};

/// Helper type distinguishing uniform (`Single`) values
/// and multiple (`Multi`) values in database queries.
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

/// Convenient wrapper for the `QueryBuilder` to
/// allow properties conversions. Represents `QueryValues::Single`.
pub struct SingleValues(pub Vec<DbKeyValue>);

/// Convenient wrapper for the `QueryBuilder` to
/// allow properties conversions. Represents `QueryValues::Multi`.
pub struct MultiValues(pub Vec<Vec<DbKeyValue>>);

/// Convenient wrapper for the `QueryBuilder` to
/// allow properties conversions. Represents list
/// of property keys.
pub struct QueryKeys(pub Vec<DbKey>);

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

impl From<Vec<DbKey>> for QueryKeys {
    fn from(value: Vec<DbKey>) -> Self {
        QueryKeys(value)
    }
}

impl<T: DbUserValue> From<&T> for MultiValues {
    fn from(value: &T) -> Self {
        MultiValues(vec![value.db_values()])
    }
}

impl<T: DbUserValue> From<&T> for SingleValues {
    fn from(value: &T) -> Self {
        SingleValues(value.db_values())
    }
}
