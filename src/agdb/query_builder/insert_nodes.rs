use super::insert_nodes_aliases::InsertNodesAliases;
use super::insert_nodes_count::InsertNodesCount;
use super::insert_nodes_values::InsertNodesValues;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryValues;
use crate::DbKeyValue;

pub struct InsertNodes(pub InsertNodesQuery);

impl InsertNodes {
    pub fn aliases(mut self, names: &[String]) -> InsertNodesAliases {
        self.0.aliases = names.to_vec();

        InsertNodesAliases(self.0)
    }

    pub fn count(mut self, num: u64) -> InsertNodesCount {
        self.0.count = num;

        InsertNodesCount(self.0)
    }

    pub fn values(mut self, key_values: &[&[DbKeyValue]]) -> InsertNodesValues {
        self.0.values = QueryValues::Multi(key_values.iter().map(|v| v.to_vec()).collect());

        InsertNodesValues(self.0)
    }

    pub fn values_id(mut self, id: QueryId) -> InsertNodesValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(vec![id]));

        InsertNodesValues(self.0)
    }

    pub fn values_ids(mut self, ids: &[QueryId]) -> InsertNodesValues {
        self.0.values = QueryValues::Ids(QueryIds::Ids(ids.to_vec()));

        InsertNodesValues(self.0)
    }

    pub fn values_uniform(mut self, key_values: &[DbKeyValue]) -> InsertNodesValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertNodesValues(self.0)
    }
}
