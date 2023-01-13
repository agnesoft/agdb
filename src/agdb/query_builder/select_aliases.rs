use super::select_aliases_of::SelectAliasesOf;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::query::select_aliases_query::SelectAliasesQuery;

pub struct SelectAliases(pub SelectAliasesQuery);

impl SelectAliases {
    pub fn of(mut self, ids: &[u64]) -> SelectAliasesOf {
        self.0.ids = QueryIds::Ids(ids.iter().map(|id| QueryId::from(*id)).collect());

        SelectAliasesOf(self.0)
    }

    pub fn search(mut self, query: SearchQuery) -> SelectAliasesOf {
        self.0.ids = QueryIds::Search(query);

        SelectAliasesOf(self.0)
    }

    pub fn query(mut self) -> SelectAliasesQuery {
        self.0.ids = QueryIds::All;

        self.0
    }
}
