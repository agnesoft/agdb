use super::insert_alias_of::InsertAliasOf;
use crate::query::{
    insert_aliases_query::InsertAliasQuery, query_id::QueryId, query_ids::QueryIds,
};

pub struct InsertAliases(pub InsertAliasQuery);

impl InsertAliases {
    pub fn of(mut self, ids: &[QueryId]) -> InsertAliasOf {
        self.0.ids = QueryIds::Ids(ids.to_vec());

        InsertAliasOf(self.0)
    }
}
