use super::insert_edge::InsertEdge;
use super::insert_edges::InsertEdges;
use super::insert_node::InsertNode;
use super::insert_nodes::InsertNodes;
use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryValues;

pub struct InsertBuilder {}

impl InsertBuilder {
    pub fn edge(self) -> InsertEdge {
        InsertEdge(InsertEdgesQuery {
            from: QueryIds::Id(0.into()),
            to: QueryIds::Id(0.into()),
            values: QueryValues::None,
            each: false,
        })
    }

    pub fn edges(self) -> InsertEdges {
        InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![]),
            to: QueryIds::Ids(vec![]),
            values: QueryValues::None,
            each: false,
        })
    }

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
