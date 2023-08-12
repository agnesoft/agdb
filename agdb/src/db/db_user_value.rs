use crate::DbError;
use crate::DbKey;
use crate::DbKeyValue;

pub trait DbUserValue: Sized {
    fn db_keys() -> Vec<DbKey>;
    fn from_db_values(values: &[DbKeyValue]) -> Result<Self, DbError>;
    fn to_db_values(&self) -> Vec<DbKeyValue>;
}
