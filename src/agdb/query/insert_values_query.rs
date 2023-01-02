use super::query_ids::QueryIds;
use super::query_values::QueryValues;
use super::Query;

pub struct InsertValuesQuery {
    pub ids: QueryIds,
    pub values: QueryValues,
}

impl Query for InsertValuesQuery {}
