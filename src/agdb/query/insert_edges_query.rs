use super::query_ids::QueryIds;
use super::query_values::QueryValues;

pub struct InsertEdgesQuery {
    pub from: QueryIds,
    pub to: QueryIds,
    pub values: QueryValues,
    pub count: u64,
    pub each: bool,
}
