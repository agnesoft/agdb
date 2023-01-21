use super::edge_count_condition::EdgeCountCondition;
use super::key_value_condition::KeyValueCondition;
use super::query_ids::QueryIds;
use crate::Comparison;
use crate::DbKey;

#[allow(dead_code)]
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
