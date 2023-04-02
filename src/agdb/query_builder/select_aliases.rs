use super::select_aliases_ids::SelectAliasesIds;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::query::select_aliases_query::SelectAliasesQuery;
use crate::query::OldQuery;

pub struct SelectAliases(pub SelectAliasesQuery);

impl SelectAliases {
    pub fn ids(mut self, ids: &[i64]) -> SelectAliasesIds {
        self.0.ids = QueryIds::Ids(ids.iter().map(|id| QueryId::from(*id)).collect());

        SelectAliasesIds(self.0)
    }

    pub fn search(mut self, query: SearchQuery) -> SelectAliasesIds {
        self.0.ids = QueryIds::Search(query);

        SelectAliasesIds(self.0)
    }

    pub fn query(mut self) -> OldQuery {
        self.0.ids = QueryIds::All;

        OldQuery::SelectAliases(self.0)
    }
}
