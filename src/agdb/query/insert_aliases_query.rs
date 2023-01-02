use super::query_ids::QueryIds;
use super::Query;

pub struct InsertAliasesQuery {
    pub ids: QueryIds,
    pub aliases: Vec<String>,
}

impl Query for InsertAliasesQuery {}
