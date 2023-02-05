use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::Query;

pub struct InsertEdgesValues(pub InsertEdgesQuery);

impl InsertEdgesValues {
    pub fn query(self) -> Query {
        Query::InsertEdges(self.0)
    }
}
