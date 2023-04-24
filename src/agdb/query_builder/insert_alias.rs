use super::insert_aliases_ids::InsertAliasesIds;
use crate::query::insert_aliases_query::InsertAliasesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;

pub struct InsertAlias(pub InsertAliasesQuery);

impl InsertAlias {
    pub fn of<T: Into<QueryId>>(mut self, id: T) -> InsertAliasesIds {
        self.0.ids = QueryIds::Ids(vec![id.into()]);

        InsertAliasesIds(self.0)
    }
}
