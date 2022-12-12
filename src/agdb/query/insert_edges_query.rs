use super::query_ids::QueryIds;
use super::query_values::QueryValues;

pub struct InsertEdgesQuery {
    pub from: QueryIds,
    pub to: QueryIds,
    pub count: u64,
    pub values: QueryValues,
}
