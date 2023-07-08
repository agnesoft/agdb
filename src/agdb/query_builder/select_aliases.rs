use crate::query::query_ids::QueryIds;
use crate::query::select_aliases_query::SelectAliasesQuery;
use crate::query::select_all_aliases_query::SelectAllAliases;

pub struct SelectAliases(pub SelectAliasesQuery);

pub struct SelectAliasesIds(pub SelectAliasesQuery);

impl SelectAliases {
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectAliasesIds {
        self.0 .0 = ids.into();

        SelectAliasesIds(self.0)
    }

    pub fn query(self) -> SelectAllAliases {
        SelectAllAliases {}
    }
}

impl SelectAliasesIds {
    pub fn query(self) -> SelectAliasesQuery {
        self.0
    }
}
