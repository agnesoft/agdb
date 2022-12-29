use crate::query::insert_edges_query::InsertEdgesQuery;

pub struct InsertEdgeValues(pub InsertEdgesQuery);

impl InsertEdgeValues {
    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }
}
