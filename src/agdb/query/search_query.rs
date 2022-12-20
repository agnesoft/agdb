use super::condition::Condition;
use super::query_id::QueryId;
use crate::DbKey;

#[allow(dead_code)]
pub enum Direction {
    From,
    To,
}

#[allow(dead_code)]
pub struct SearchQuery {
    origin: QueryId,
    direction: Direction,
    limit: u64,
    offset: u64,
    order_by: Vec<DbKey>,
    conditions: Vec<Condition>,
}
