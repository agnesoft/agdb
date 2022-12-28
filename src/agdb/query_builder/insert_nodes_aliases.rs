use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::query_values::QueryValues;
use crate::DbKeyValue;
use crate::Query;

use super::insert_nodes_values::InsertNodesValues;

pub struct InsertNodesAliases(pub InsertNodesQuery);

impl InsertNodesAliases {
    pub fn query(self) -> Query {
        Query::InsertNodes(self.0)
    }

    pub fn values(mut self, key_values: &[&[DbKeyValue]]) -> InsertNodesValues {
        self.0.values = QueryValues::Multi(key_values.iter().map(|v| v.to_vec()).collect());

        InsertNodesValues(self.0)
    }
}
