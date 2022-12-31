use super::insert_edges_from::InsertEdgesFrom;
use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;

pub struct InsertEdges(pub InsertEdgesQuery);

impl InsertEdges {
    pub fn from(mut self, ids: &[QueryId]) -> InsertEdgesFrom {
        self.0.from = QueryIds::Ids(ids.to_vec());

        InsertEdgesFrom(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_query(mut self, query: SearchQuery) -> InsertEdgesFrom {
        self.0.from = QueryIds::Search(query);

        InsertEdgesFrom(self.0)
    }
}
