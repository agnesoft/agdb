use crate::db::db_element::DbElement;
use crate::DbId;

#[derive(Debug, Default)]
pub struct QueryResult {
    pub result: i64,
    pub elements: Vec<DbElement>,
}

impl QueryResult {
    pub fn ids(&self) -> Vec<DbId> {
        self.elements.iter().map(|e| e.id).collect()
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
