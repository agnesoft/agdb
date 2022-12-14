use super::insert_nodes_values::InsertNodesValues;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryValues;
use crate::DbKeyValue;

pub struct InsertNodesCount(pub InsertNodesQuery);

impl InsertNodesCount {
    pub fn query(self) -> InsertNodesQuery {
        self.0
    }

    pub fn values_id(mut self, id: QueryId) -> InsertNodesValues {
        self.0.values = QueryValues::Ids(QueryIds::Id(id));

        InsertNodesValues(self.0)
    }

    pub fn values_uniform(mut self, key_values: &[DbKeyValue]) -> InsertNodesValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertNodesValues(self.0)
    }
}
