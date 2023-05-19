use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::query::select_key_count_query::SelectKeyCountQuery;

pub struct SelectKeyCount(pub SelectKeyCountQuery);

pub struct SelectKeyCountIds(pub SelectKeyCountQuery);

impl SelectKeyCount {
    pub fn id(mut self, id: QueryId) -> SelectKeyCountIds {
        self.0 .0 = QueryIds::Ids(vec![id]);

        SelectKeyCountIds(self.0)
    }

    pub fn ids(mut self, ids: &[QueryId]) -> SelectKeyCountIds {
        self.0 .0 = QueryIds::Ids(ids.to_vec());

        SelectKeyCountIds(self.0)
    }

    pub fn search(mut self, query: SearchQuery) -> SelectKeyCountIds {
        self.0 .0 = QueryIds::Search(query);

        SelectKeyCountIds(self.0)
    }
}

impl SelectKeyCountIds {
    pub fn query(self) -> SelectKeyCountQuery {
        self.0
    }
}
