use crate::query::insert_aliases_query::InsertAliasesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;

pub struct InsertAlias(pub InsertAliasesQuery);

pub struct InsertAliasesIds(pub InsertAliasesQuery);

pub struct InsertAliases(pub InsertAliasesQuery);

impl InsertAlias {
    pub fn of<T: Into<QueryId>>(mut self, id: T) -> InsertAliasesIds {
        self.0.ids = QueryIds::Ids(vec![id.into()]);

        InsertAliasesIds(self.0)
    }
}

impl InsertAliases {
    pub fn of(mut self, ids: &[QueryId]) -> InsertAliasesIds {
        self.0.ids = QueryIds::Ids(ids.to_vec());

        InsertAliasesIds(self.0)
    }
}

impl InsertAliasesIds {
    pub fn query(self) -> InsertAliasesQuery {
        self.0
    }
}
