use super::insert_aliases_ids::InsertAliasesIds;
use crate::query::insert_aliases_query::InsertAliasesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;

pub struct InsertAlias(pub InsertAliasesQuery);

impl InsertAlias {
    pub fn id(mut self, id: QueryId) -> InsertAliasesIds {
        self.0.ids = QueryIds::Id(id);

        InsertAliasesIds(self.0)
    }
}
