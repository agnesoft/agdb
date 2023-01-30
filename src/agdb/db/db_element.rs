use super::db_key_value::DbKeyValue;

#[derive(Debug, PartialEq)]
pub struct DbElement {
    pub index: u64,
    pub values: Vec<DbKeyValue>,
}
