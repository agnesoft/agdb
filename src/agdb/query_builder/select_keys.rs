use super::select_keys_ids::SelectKeysIds;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::query::select_keys_query::SelectKeysQuery;

pub struct SelectKeys(pub SelectKeysQuery);

impl SelectKeys {
    pub fn id(mut self, id: QueryId) -> SelectKeysIds {
        self.0 .0 = QueryIds::Id(id);

        SelectKeysIds(self.0)
    }

    pub fn ids(mut self, ids: &[QueryId]) -> SelectKeysIds {
        self.0 .0 = QueryIds::Ids(ids.to_vec());

        SelectKeysIds(self.0)
    }

    pub fn search(mut self, query: SearchQuery) -> SelectKeysIds {
        self.0 .0 = QueryIds::Search(query);

        SelectKeysIds(self.0)
    }
}
