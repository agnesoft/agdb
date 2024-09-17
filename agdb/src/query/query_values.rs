use crate::DbKeyValue;
use crate::DbUserValue;

/// Helper type distinguishing uniform (`Single`) values
/// and multiple (`Multi`) values in database queries.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[derive(Debug, PartialEq)]
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

impl From<Vec<DbKeyValue>> for SingleValues {
    fn from(values: Vec<DbKeyValue>) -> Self {
        SingleValues(values)
    }
}

impl From<&Vec<DbKeyValue>> for SingleValues {
    fn from(values: &Vec<DbKeyValue>) -> Self {
        SingleValues(values.clone())
    }
}

impl From<&[DbKeyValue]> for SingleValues {
    fn from(values: &[DbKeyValue]) -> Self {
        SingleValues(values.to_vec())
    }
}

impl<const N: usize> From<[DbKeyValue; N]> for SingleValues {
    fn from(values: [DbKeyValue; N]) -> Self {
        SingleValues(values.to_vec())
    }
}

impl<T: DbUserValue> From<T> for SingleValues {
    fn from(value: T) -> Self {
        SingleValues(value.to_db_values())
    }
}

impl From<Vec<Vec<DbKeyValue>>> for MultiValues {
    fn from(values: Vec<Vec<DbKeyValue>>) -> Self {
        MultiValues(values)
    }
}

impl From<&Vec<Vec<DbKeyValue>>> for MultiValues {
    fn from(values: &Vec<Vec<DbKeyValue>>) -> Self {
        MultiValues(values.clone())
    }
}

impl From<Vec<&[DbKeyValue]>> for MultiValues {
    fn from(values: Vec<&[DbKeyValue]>) -> Self {
        MultiValues(values.into_iter().map(|v| v.to_vec()).collect())
    }
}

impl From<&Vec<&[DbKeyValue]>> for MultiValues {
    fn from(values: &Vec<&[DbKeyValue]>) -> Self {
        MultiValues(values.iter().map(|v| v.to_vec()).collect())
    }
}

impl From<&[Vec<DbKeyValue>]> for MultiValues {
    fn from(values: &[Vec<DbKeyValue>]) -> Self {
        MultiValues(values.to_vec())
    }
}

impl From<&[&[DbKeyValue]]> for MultiValues {
    fn from(values: &[&[DbKeyValue]]) -> Self {
        MultiValues(values.iter().map(|v| v.to_vec()).collect())
    }
}

impl<const N: usize> From<[Vec<DbKeyValue>; N]> for MultiValues {
    fn from(values: [Vec<DbKeyValue>; N]) -> Self {
        MultiValues(values.into_iter().collect())
    }
}

impl<const N: usize> From<[&[DbKeyValue]; N]> for MultiValues {
    fn from(values: [&[DbKeyValue]; N]) -> Self {
        MultiValues(values.into_iter().map(|v| v.to_vec()).collect())
    }
}

impl<const N: usize, const N2: usize> From<[[DbKeyValue; N2]; N]> for MultiValues {
    fn from(values: [[DbKeyValue; N2]; N]) -> Self {
        MultiValues(values.into_iter().map(|v| v.to_vec()).collect())
    }
}

impl<T: DbUserValue> From<T> for MultiValues {
    fn from(value: T) -> Self {
        MultiValues(vec![value.to_db_values()])
    }
}

impl<T: DbUserValue> From<&[T]> for MultiValues {
    fn from(value: &[T]) -> Self {
        MultiValues(value.iter().map(|v| v.to_db_values()).collect())
    }
}

impl<T: DbUserValue, const N: usize> From<[T; N]> for MultiValues {
    fn from(value: [T; N]) -> Self {
        MultiValues(value.iter().map(|v| v.to_db_values()).collect())
    }
}

impl<T: DbUserValue> From<&Vec<T>> for MultiValues {
    fn from(value: &Vec<T>) -> Self {
        MultiValues(value.iter().map(|v| v.to_db_values()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_values() {
        let _values = MultiValues::from(vec![vec![("k", 1).into()]]);
        let _values = MultiValues::from(&vec![vec![("k", 1).into()]]);
        let _values = MultiValues::from(vec![[("k", 1).into()].as_slice()]);
        let _values = MultiValues::from(&vec![[("k", 1).into()].as_slice()]);

        let _values = MultiValues::from([vec![("k", 1).into()]].as_slice());
        let _values = MultiValues::from([[("k", 1).into()].as_slice()].as_slice());

        let _values = MultiValues::from([vec![("k", 1).into()]]);
        let _values = MultiValues::from([[("k", 1).into()].as_slice()]);
        let _values = MultiValues::from([[("k", 1).into()]]);
    }

    #[test]
    fn single_values() {
        let _values = SingleValues::from(vec![("k", 1).into()]);
        let _values = SingleValues::from(&vec![("k", 1).into()]);
        let _values = SingleValues::from([("k", 1).into()].as_slice());
        let _values = SingleValues::from([("k", 1).into()]);
    }
}
