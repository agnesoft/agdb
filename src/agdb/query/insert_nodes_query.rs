use super::query_values::QueryValues;
use super::Query;

pub struct InsertNodesQuery {
    pub count: u64,
    pub values: QueryValues,
    pub aliases: Vec<String>,
}

impl Query for InsertNodesQuery {}
