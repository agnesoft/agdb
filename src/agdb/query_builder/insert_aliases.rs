use crate::query::insert_aliases_query::InsertAliasesQuery;
use crate::query::query_ids::QueryIds;

pub struct InsertAliases(pub InsertAliasesQuery);

pub struct InsertAliasesIds(pub InsertAliasesQuery);

impl InsertAliases {
    pub fn of<T: Into<QueryIds>>(mut self, ids: T) -> InsertAliasesIds {
        self.0.ids = ids.into();

        InsertAliasesIds(self.0)
    }
}

impl InsertAliasesIds {
    pub fn query(self) -> InsertAliasesQuery {
        self.0
    }
}
