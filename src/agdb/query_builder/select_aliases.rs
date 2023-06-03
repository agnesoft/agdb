use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::query::select_aliases_query::SelectAliasesQuery;
use crate::query::select_all_aliases_query::SelectAllAliases;

pub struct SelectAliases(pub SelectAliasesQuery);

pub struct SelectAliasesIds(pub SelectAliasesQuery);

impl SelectAliasesIds {
    pub fn query(self) -> SelectAliasesQuery {
        self.0
    }
}

impl SelectAliases {
    pub fn ids(mut self, ids: &[QueryId]) -> SelectAliasesIds {
        self.0.ids = QueryIds::Ids(ids.to_vec());

        SelectAliasesIds(self.0)
    }

    pub fn search(mut self, query: SearchQuery) -> SelectAliasesIds {
        self.0.ids = QueryIds::Search(query);

        SelectAliasesIds(self.0)
    }

    pub fn query(self) -> SelectAllAliases {
        SelectAllAliases {}
    }
}
