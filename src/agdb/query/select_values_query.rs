use crate::DbKey;

use super::query_ids::QueryIds;

pub struct SelectValuesQuery {
    pub keys: Vec<DbKey>,
    pub ids: QueryIds,
}
