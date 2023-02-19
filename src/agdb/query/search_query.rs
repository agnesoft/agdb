use super::condition::Condition;
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
    pub conditions: Vec<Condition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            SearchQuery {
                origin: QueryId::Id(0),
                destination: QueryId::Id(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![]
            }
        );
    }

    #[test]
    fn derived_from_clone() {
        let left = SearchQuery {
            origin: QueryId::Id(0),
            destination: QueryId::Id(0),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        };
        let right = left.clone();

        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            SearchQuery {
                origin: QueryId::Id(0),
                destination: QueryId::Id(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![]
            },
            SearchQuery {
                origin: QueryId::Id(0),
                destination: QueryId::Id(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![]
            }
        );
    }
}
