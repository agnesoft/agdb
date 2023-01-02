use super::insert_aliases_of::InsertAliasesOf;
use crate::query::insert_aliases_query::InsertAliasesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;

pub struct InsertAlias(pub InsertAliasesQuery);

impl InsertAlias {
    pub fn of(mut self, id: QueryId) -> InsertAliasesOf {
        self.0.ids = QueryIds::Id(id);

        InsertAliasesOf(self.0)
    }
}
