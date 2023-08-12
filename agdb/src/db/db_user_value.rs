use crate::DbElement;
use crate::DbError;
use crate::DbId;
use crate::DbKey;
use crate::DbKeyValue;

pub trait DbUserValue: Sized {
    fn db_id(&self) -> Option<DbId>;
    fn db_keys() -> Vec<DbKey>;
    fn from_db_element(element: &DbElement) -> Result<Self, DbError>;
    fn to_db_values(&self) -> Vec<DbKeyValue>;
}
