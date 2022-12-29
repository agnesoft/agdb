use crate::query::direction::Direction;
use crate::query::query_id::QueryId;
use crate::query::search_query::SearchQuery;

use super::select_from::SelectFrom;

pub struct Select {}

impl Select {
    pub fn from(self, origin: QueryId) -> SelectFrom {
        SelectFrom(SearchQuery {
            origin,
            direction: Direction::From,
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
    }
}
