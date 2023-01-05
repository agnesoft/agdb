use super::select_limit::SelectLimit;
use crate::query::search_query::SearchQuery;

pub struct SelectOffset(pub SearchQuery);

impl SelectOffset {
    pub fn limit(mut self, value: u64) -> SelectLimit {
        self.0.limit = value;

        SelectLimit(self.0)
    }

    pub fn query(self) -> SearchQuery {
        self.0
    }
}
