use crate::db::db_element::DbElement;

#[derive(Debug, Default)]
pub struct QueryResult {
    pub result: i64,
    pub elements: Vec<DbElement>,
}
