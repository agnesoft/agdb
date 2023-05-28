use super::db_key_value::DbKeyValue;
use crate::DbId;

#[derive(Debug, PartialEq)]
pub struct DbElement {
    pub index: DbId,
    pub values: Vec<DbKeyValue>,
}
