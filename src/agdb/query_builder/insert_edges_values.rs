use crate::query::insert_edges_query::InsertEdgesQuery;

pub struct InsertEdgesValues(pub InsertEdgesQuery);

impl InsertEdgesValues {
    pub fn query(self) -> InsertEdgesQuery {
        self.0
    }
}
