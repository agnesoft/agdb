use super::query_condition::QueryCondition;
use super::query_id::QueryId;
use crate::DbKey;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SearchQuery {
    pub origin: QueryId,
    pub destination: QueryId,
    pub limit: u64,
    pub offset: u64,
    pub order_by: Vec<DbKey>,
    pub conditions: Vec<QueryCondition>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            SearchQuery {
                origin: QueryId::from(0),
                destination: QueryId::from(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![]
            }
        );
    }
}
