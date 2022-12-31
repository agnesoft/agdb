use super::insert_edges_from_to::InsertEdgesFromTo;
use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;

pub struct InsertEdgesFrom(pub InsertEdgesQuery);

impl InsertEdgesFrom {
    pub fn to(mut self, ids: &[QueryId]) -> InsertEdgesFromTo {
        self.0.to = QueryIds::Ids(ids.to_vec());

        InsertEdgesFromTo(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn to_query(mut self, query: SearchQuery) -> InsertEdgesFromTo {
        self.0.to = QueryIds::Search(query);

        InsertEdgesFromTo(self.0)
    }
}
