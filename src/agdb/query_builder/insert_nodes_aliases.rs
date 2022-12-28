use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
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

    pub fn values_id(mut self, id: QueryId) -> InsertNodesValues {
        self.0.values = QueryValues::Query(QueryIds::Id(id));

        InsertNodesValues(self.0)
    }

    pub fn values_ids(mut self, ids: &[QueryId]) -> InsertNodesValues {
        self.0.values = QueryValues::Query(QueryIds::Ids(ids.to_vec()));

        InsertNodesValues(self.0)
    }
}
