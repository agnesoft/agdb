use crate::DbKey;

use super::query_ids::QueryIds;

pub struct SelectQuery {
    pub keys: Vec<DbKey>,
    pub ids: QueryIds,
}
