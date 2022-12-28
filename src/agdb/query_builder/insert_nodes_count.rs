use super::insert_nodes_aliases::InsertNodesAliases;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::Query;

pub struct InsertNodesCount(pub InsertNodesQuery);

impl InsertNodesCount {
    pub fn aliases(mut self, names: &[String]) -> InsertNodesAliases {
        self.0.aliases = names.to_vec();

        InsertNodesAliases(self.0)
    }

    pub fn query(self) -> Query {
        Query::InsertNodes(self.0)
    }
}
