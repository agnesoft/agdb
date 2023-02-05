use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::Query;

pub struct InsertEdgeValues(pub InsertEdgesQuery);

impl InsertEdgeValues {
    pub fn query(self) -> Query {
        Query::InsertEdges(self.0)
    }
}
