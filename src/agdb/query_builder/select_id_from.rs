use super::select_id_to::SelectIdTo;
use super::select_limit::SelectLimit;
use crate::query::query_id::QueryId;
use crate::query::search_query::SearchQuery;

pub struct SelectIdFrom(pub SearchQuery);

impl SelectIdFrom {
    pub fn offset(mut self, value: u64) -> SelectLimit {
        self.0.offset = value;

        SelectLimit(self.0)
    }

    pub fn query(self) -> SearchQuery {
        self.0
    }

    pub fn to(mut self, id: QueryId) -> SelectIdTo {
        self.0.destination = id;

        SelectIdTo(self.0)
    }
}
