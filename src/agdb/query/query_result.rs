use crate::db::db_element::DbElement;
use crate::DbId;

/// Universal database result. Successful
/// execution of a query will always yield
/// this type. The `result` field is a numerical
/// representation of the result while the
/// `elements` are the list of `DbElement`s
/// with database ids and properties (key-value pairs).
#[derive(Debug, Default)]
pub struct QueryResult {
    /// Query result
    pub result: i64,

    /// List of elements yielded by the query
    /// possibly with a list of properties.
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
