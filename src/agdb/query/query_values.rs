use crate::db::db_key_value::DbKeyValue;

pub enum QueryValues {
    Single(Vec<DbKeyValue>),
    Multi(Vec<Vec<DbKeyValue>>),
}
