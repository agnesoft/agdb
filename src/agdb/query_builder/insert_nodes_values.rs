use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::Query;

pub struct InsertNodesValues(pub InsertNodesQuery);

impl InsertNodesValues {
    pub fn query(self) -> Query {
        Query::InsertNodes(self.0)
    }
}
