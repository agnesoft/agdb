use crate::query::insert_nodes_query::InsertNodesQuery;

pub struct InsertNodesValues(pub InsertNodesQuery);

impl InsertNodesValues {
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }
}
