use super::insert_alias_of::InsertAliasOf;
use crate::query::insert_aliases_query::InsertAliasQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;

pub struct InsertAlias(pub InsertAliasQuery);

impl InsertAlias {
    pub fn of(mut self, id: QueryId) -> InsertAliasOf {
        self.0.ids = QueryIds::Id(id);

        InsertAliasOf(self.0)
    }
}
