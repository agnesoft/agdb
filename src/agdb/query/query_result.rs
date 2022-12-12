use crate::db::db_element::DbElement;

#[derive(Default)]
pub struct QueryResult {
    pub result: u64,
    pub elements: Vec<DbElement>,
}
