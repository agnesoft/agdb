use super::select_limit::SelectLimit;
use super::select_offset::SelectOffset;
use crate::query::search_query::SearchQuery;

pub struct SearchTo(pub SearchQuery);

impl SearchTo {
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
}
