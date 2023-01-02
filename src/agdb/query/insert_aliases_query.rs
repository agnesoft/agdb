use super::query_ids::QueryIds;
use super::Query;

pub struct InsertAliasQuery {
    pub ids: QueryIds,
    pub aliases: Vec<String>,
}

impl Query for InsertAliasQuery {}
