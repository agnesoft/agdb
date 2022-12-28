use super::insert_node::InsertNode;
use super::insert_nodes::InsertNodes;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::query_values::QueryValues;

pub struct InsertBuilder {}

impl InsertBuilder {
    pub fn node(self) -> InsertNode {
        InsertNode(InsertNodesQuery {
            count: 1,
            values: QueryValues::None,
            aliases: vec![],
        })
    }

    pub fn nodes(self) -> InsertNodes {
        InsertNodes(InsertNodesQuery {
            count: 0,
            values: QueryValues::None,
            aliases: vec![],
        })
    }
}
