use super::db_value::DbValue;

pub type DbKey = DbValue;

#[derive(Debug, Clone, PartialEq)]
pub enum DbKeyOrder {
    Asc(DbKey),
    Desc(DbKey),
}
