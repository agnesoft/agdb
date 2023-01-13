use super::select_keys_from::SelectKeysFrom;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::query::select_keys_query::SelectKeysQuery;

pub struct SelectKeys(pub SelectKeysQuery);

impl SelectKeys {
    #[allow(clippy::wrong_self_convention)]
    pub fn from(mut self, id: QueryId) -> SelectKeysFrom {
        self.0 .0 = QueryIds::Id(id);

        SelectKeysFrom(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_ids(mut self, ids: &[QueryId]) -> SelectKeysFrom {
        self.0 .0 = QueryIds::Ids(ids.to_vec());

        SelectKeysFrom(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_search(mut self, query: SearchQuery) -> SelectKeysFrom {
        self.0 .0 = QueryIds::Search(query);

        SelectKeysFrom(self.0)
    }
}
