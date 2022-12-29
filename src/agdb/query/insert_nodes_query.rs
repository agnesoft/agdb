use super::query_values::QueryValues;
use super::Query;
use crate::QueryData;

pub struct InsertNodesQuery {
    pub count: u64,
    pub values: QueryValues,
    pub aliases: Vec<String>,
}

impl Query for InsertNodesQuery {
    fn data(self) -> QueryData {
        QueryData::InsertNodes(self)
    }
}
