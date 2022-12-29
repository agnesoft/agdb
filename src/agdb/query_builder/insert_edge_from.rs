use super::insert_edge_from_to::InsertEdgeFromTo;
use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;

pub struct InsertEdgeFrom(pub InsertEdgesQuery);

impl InsertEdgeFrom {
    pub fn to(mut self, id: QueryId) -> InsertEdgeFromTo {
        self.0.to = QueryIds::Id(id);

        InsertEdgeFromTo(self.0)
    }
}
