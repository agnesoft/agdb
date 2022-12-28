use super::insert_node_alias::InsertNodeAlias;
use super::insert_node_values::InsertNodeValues;
use crate::db::db_key_value::DbKeyValue;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryValues;
use crate::Query;

pub struct InsertNode(pub InsertNodesQuery);

impl InsertNode {
    pub fn alias(mut self, name: &str) -> InsertNodeAlias {
        self.0.aliases.push(name.to_string());

        InsertNodeAlias(self.0)
    }

    pub fn values(mut self, key_values: &[DbKeyValue]) -> InsertNodeValues {
        self.0.values = QueryValues::Single(key_values.to_vec());

        InsertNodeValues(self.0)
    }

    pub fn values_id(mut self, id: QueryId) -> InsertNodeValues {
        self.0.values = QueryValues::Query(QueryIds::Id(id));

        InsertNodeValues(self.0)
    }

    pub fn query(self) -> Query {
        Query::InsertNodes(self.0)
    }
}
