use super::query_ids::QueryIds;
use super::Query;

pub struct SelectAliasesQuery {
    pub ids: QueryIds,
}

impl Query for SelectAliasesQuery {}
