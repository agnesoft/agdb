use crate::DbKey;

use super::query_ids::QueryIds;

pub struct SelectKeysQuery {
    pub keys: Vec<DbKey>,
    pub ids: QueryIds,
}
