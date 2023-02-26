use super::insert_aliases_ids::InsertAliasesIds;
use crate::query::insert_aliases_query::InsertAliasesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;

pub struct InsertAliases(pub InsertAliasesQuery);

impl InsertAliases {
    pub fn ids(mut self, ids: &[QueryId]) -> InsertAliasesIds {
        self.0.ids = QueryIds::Ids(ids.to_vec());

        InsertAliasesIds(self.0)
    }
}
