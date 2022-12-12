use crate::db::db_key_value::DbKeyValue;
use crate::Query;

#[allow(dead_code)]
pub enum QueryValues {
    None,
    Values(Vec<Vec<DbKeyValue>>),
    Query(Box<Query>),
}
