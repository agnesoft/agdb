use super::query_ids::QueryIds;
use crate::db::db_key_value::DbKeyValue;

pub enum QueryValues {
    None,
    Ids(QueryIds),
    Single(Vec<DbKeyValue>),
    Multi(Vec<Vec<DbKeyValue>>),
}
