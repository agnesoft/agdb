use super::distance_condition::DistanceCondition;
use super::edge_count_condition::EdgeCountCondition;
use super::key_value_condition::KeyValueCondition;
use super::query_ids::QueryIds;
use crate::DbKey;

#[allow(dead_code)]
pub enum Condition {
    And,
    Distance(DistanceCondition),
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
