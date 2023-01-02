use super::query_ids::QueryIds;
use super::query_values::QueryValues;
use super::Query;

pub struct InsertEdgesQuery {
    pub from: QueryIds,
    pub to: QueryIds,
    pub values: QueryValues,
    pub each: bool,
}

impl Query for InsertEdgesQuery {}
