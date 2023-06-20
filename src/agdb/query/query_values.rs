use crate::{db::db_key_value::DbKeyValue, DbKey};

pub enum QueryValues {
    Single(Vec<DbKeyValue>),
    Multi(Vec<Vec<DbKeyValue>>),
}

pub struct SingleValues(pub Vec<DbKeyValue>);
pub struct MultiValues(pub Vec<Vec<DbKeyValue>>);

pub struct QueryKeys(pub Vec<DbKey>);

impl From<Vec<DbKeyValue>> for SingleValues {
    fn from(values: Vec<DbKeyValue>) -> Self {
        SingleValues(values)
    }
}

impl From<&[DbKeyValue]> for SingleValues {
    fn from(values: &[DbKeyValue]) -> Self {
        SingleValues(values.to_vec())
    }
}

impl From<Vec<Vec<DbKeyValue>>> for MultiValues {
    fn from(values: Vec<Vec<DbKeyValue>>) -> Self {
        MultiValues(values)
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

impl From<Vec<DbKey>> for QueryKeys {
    fn from(value: Vec<DbKey>) -> Self {
        QueryKeys(value)
    }
}

impl From<&[DbKey]> for QueryKeys {
    fn from(value: &[DbKey]) -> Self {
        QueryKeys(value.to_vec())
    }
}
