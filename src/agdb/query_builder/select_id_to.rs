use super::select_limit::SelectLimit;
use crate::query::search_query::SearchQuery;

pub struct SelectIdTo(pub SearchQuery);

impl SelectIdTo {
    pub fn query(self) -> SearchQuery {
        self.0
    }

    pub fn offset(mut self, value: u64) -> SelectLimit {
        self.0.offset = value;

        SelectLimit(self.0)
    }
}
