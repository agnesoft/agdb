use crate::DbError;
use crate::DbKey;
use crate::DbKeyValue;

pub trait DbUserValue: Sized {
    fn from_db_values(values: &[DbKeyValue]) -> Result<Self, DbError>;
    fn db_values(&self) -> Vec<DbKeyValue>;
    fn db_keys(&self) -> Vec<DbKey>;
}
