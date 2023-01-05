use crate::query::query_id::QueryId;
use crate::query::search_query::SearchQuery;

pub struct SelectIdsFromTo(pub SearchQuery);

impl SelectIdsFromTo {
    pub fn query(self) -> SearchQuery {
        self.0
    }

    pub fn offset(self, value: u64) -> SelectIdsFromToOffset {}
}
