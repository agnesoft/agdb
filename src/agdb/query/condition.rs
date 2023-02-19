use super::edge_count_condition::EdgeCountCondition;
use super::key_value_condition::KeyValueCondition;
use super::query_ids::QueryIds;
use crate::Comparison;
use crate::DbKey;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
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
    fn derived_from_debug() {
        format!("{:?}", Condition::Where);
    }

    #[test]
    fn derived_from_clone() {
        let left = Condition::Where;
        let right = left.clone();

        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(Condition::Where, Condition::Where);
    }
}
