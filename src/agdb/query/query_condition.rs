pub(crate) mod comparison;
pub(crate) mod direction;
pub(crate) mod edge_count_condition;
pub(crate) mod key_value_condition;

use super::query_ids::QueryIds;
use crate::Comparison;
use crate::DbKey;
use edge_count_condition::EdgeCountCondition;
use key_value_condition::KeyValueCondition;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum QueryCondition {
    And,
    Distance(Comparison),
    Edge,
    EdgeCount(EdgeCountCondition),
    EndWhere,
    Ids(QueryIds),
    KeyValue(KeyValueCondition),
    Keys(Vec<DbKey>),
    Node,
    Not,
    NotBeyond,
    Or,
    Where,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_clone() {
        let left = QueryCondition::Where;
        let right = left.clone();
        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(QueryCondition::Where, QueryCondition::Where);
    }

    #[test]
    fn derived_from_debug() {
        format!("{:?}", QueryCondition::Where);
    }
}
