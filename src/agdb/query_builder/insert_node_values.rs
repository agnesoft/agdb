use crate::query::insert_nodes_query::InsertNodesQuery;

pub struct InsertNodeValues(pub InsertNodesQuery);

impl InsertNodeValues {
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }
}
