use super::search_to::SearchTo;
use super::select_limit::SelectLimit;
use super::select_offset::SelectOffset;
use crate::query::query_id::QueryId;
use crate::query::search_query::SearchQuery;

pub struct SearchFrom(pub SearchQuery);

impl SearchFrom {
    pub fn limit(mut self, value: u64) -> SelectLimit {
        self.0.limit = value;

        SelectLimit(self.0)
    }

    pub fn offset(mut self, value: u64) -> SelectOffset {
        self.0.offset = value;

        SelectOffset(self.0)
    }

    pub fn query(self) -> SearchQuery {
        self.0
    }

    pub fn to(mut self, id: QueryId) -> SearchTo {
        self.0.destination = id;

        SearchTo(self.0)
    }
}
