use crate::query::search_query::SearchQuery;

pub struct SelectFrom(pub SearchQuery);

impl SelectFrom {
    pub fn query(self) -> SearchQuery {
        self.0
    }
}
