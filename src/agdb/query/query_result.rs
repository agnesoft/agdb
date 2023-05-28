use crate::db::db_element::DbElement;

#[derive(Debug, Default)]
pub struct QueryResult {
    pub result: i64,
    pub elements: Vec<DbElement>,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn derived_from_debug() {
        format!("{:?}", QueryResult::default());
    }
}
