use super::search_from::SearchFrom;
use super::search_to::SearchTo;
use crate::query::query_id::QueryId;
use crate::query::search_query::SearchQuery;

pub struct Search {}

impl Search {
    pub fn from(self, id: QueryId) -> SearchFrom {
        SearchFrom(SearchQuery {
            origin: id,
            destination: QueryId::Id(0),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
    }

    pub fn to(self, id: QueryId) -> SearchTo {
        SearchTo(SearchQuery {
            origin: QueryId::Id(0),
            destination: id,
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
    }
}
