use super::insert_edge_from::InsertEdgeFrom;
use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;

pub struct InsertEdge(pub InsertEdgesQuery);

impl InsertEdge {
    pub fn from(mut self, id: QueryId) -> InsertEdgeFrom {
        self.0.from = QueryIds::Ids(vec![id]);

        InsertEdgeFrom(self.0)
    }
}
