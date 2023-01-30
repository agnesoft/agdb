use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::Query;

pub struct InsertNodeValues(pub InsertNodesQuery);

impl InsertNodeValues {
    pub fn query(self) -> Query {
        Query::InsertNodes(self.0)
    }
}
