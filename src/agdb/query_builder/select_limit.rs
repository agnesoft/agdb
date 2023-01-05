use crate::query::search_query::SearchQuery;

pub struct SelectLimit(pub SearchQuery);

impl SelectLimit {
    pub fn query(self) -> SearchQuery {
        self.0
    }
}
