use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::query_values::QueryValues;

use super::insert_node::InsertNodeBuilder;

pub struct InsertBuilder {}

impl InsertBuilder {
    pub fn node(self) -> InsertNodeBuilder {
        InsertNodeBuilder(InsertNodesQuery {
            count: 1,
            values: QueryValues::None,
            alias: String::new(),
        })
    }
}
