use super::condition::Condition;
use super::query_id::QueryId;
use crate::DbKey;

#[allow(dead_code)]
pub struct SearchQuery {
    pub origin: QueryId,
    pub destination: QueryId,
    pub limit: u64,
    pub offset: u64,
    pub order_by: Vec<DbKey>,
    pub conditions: Vec<Condition>,
}
