use super::select_key_count_from::SelectKeyCountFrom;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::query::select_key_count_query::SelectKeyCountQuery;

pub struct SelectKeyCount(pub SelectKeyCountQuery);

impl SelectKeyCount {
    #[allow(clippy::wrong_self_convention)]
    pub fn from(mut self, id: QueryId) -> SelectKeyCountFrom {
        self.0 .0 = QueryIds::Id(id);

        SelectKeyCountFrom(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_ids(mut self, ids: &[QueryId]) -> SelectKeyCountFrom {
        self.0 .0 = QueryIds::Ids(ids.to_vec());

        SelectKeyCountFrom(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_search(mut self, query: SearchQuery) -> SelectKeyCountFrom {
        self.0 .0 = QueryIds::Search(query);

        SelectKeyCountFrom(self.0)
    }
}
