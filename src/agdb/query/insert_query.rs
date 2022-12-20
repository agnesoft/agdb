use super::query_ids::QueryIds;
use super::query_values::QueryValues;

pub struct InsertQuery {
    pub ids: QueryIds,
    pub values: QueryValues,
}
