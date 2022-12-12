use super::query_ids::QueryIds;
use super::query_values::QueryValues;

pub struct InsertValuesQuery {
    pub ids: QueryIds,
    pub values: QueryValues,
}
