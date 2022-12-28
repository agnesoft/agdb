use super::insert_node_values::InsertNodeValues;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryValues;
use crate::DbKeyValue;
use crate::Query;

pub struct InsertNodeAlias(pub InsertNodesQuery);

impl InsertNodeAlias {
    pub fn query(self) -> Query {
        Query::InsertNodes(self.0)
    }

    pub fn values(mut self, key_values: &[DbKeyValue]) -> InsertNodeValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertNodeValues(self.0)
    }

    pub fn values_id(mut self, id: QueryId) -> InsertNodeValues {
        self.0.values = QueryValues::Query(QueryIds::Ids(vec![id]));

        InsertNodeValues(self.0)
    }
}
