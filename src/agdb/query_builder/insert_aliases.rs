use super::insert_aliases_of::InsertAliasesOf;
use crate::query::{
    insert_aliases_query::InsertAliasesQuery, query_id::QueryId, query_ids::QueryIds,
};

pub struct InsertAliases(pub InsertAliasesQuery);

impl InsertAliases {
    pub fn of(mut self, ids: &[QueryId]) -> InsertAliasesOf {
        self.0.ids = QueryIds::Ids(ids.to_vec());

        InsertAliasesOf(self.0)
    }
}
