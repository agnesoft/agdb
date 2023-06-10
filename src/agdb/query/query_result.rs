use super::query_id::QueryId;
use crate::db::db_element::DbElement;

#[derive(Debug, Default)]
pub struct QueryResult {
    pub result: i64,
    pub elements: Vec<DbElement>,
}

impl QueryResult {
    pub fn ids(&self) -> Vec<QueryId> {
        self.elements.iter().map(|e| QueryId::Id(e.id)).collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn derived_from_debug() {
        format!("{:?}", QueryResult::default());
    }
}
