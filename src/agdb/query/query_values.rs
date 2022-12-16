use super::query_ids::QueryIds;
use crate::db::db_key_value::DbKeyValue;

#[allow(dead_code)]
pub enum QueryValues {
    None,
    Values(Vec<Vec<DbKeyValue>>),
    Query(QueryIds),
}
