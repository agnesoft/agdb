use super::search_from::SearchFrom;
use super::search_to::SearchTo;
use crate::query::query_id::QueryId;
use crate::query::search_query::SearchQuery;

pub struct Search(pub SearchQuery);

impl Search {
    pub fn from(mut self, id: QueryId) -> SearchFrom {
        self.0.origin = id;

        SearchFrom(self.0)
    }

    pub fn to(mut self, id: QueryId) -> SearchTo {
        self.0.destination = id;

        SearchTo(self.0)
    }
}
