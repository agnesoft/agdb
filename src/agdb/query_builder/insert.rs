use super::insert_alias::InsertAlias;
use super::insert_aliases::InsertAliases;
use super::insert_edge::InsertEdge;
use super::insert_edges::InsertEdges;
use super::insert_node::InsertNode;
use super::insert_nodes::InsertNodes;
use super::insert_values::InsertValues;
use super::insert_values_multi::InsertValuesMulti;
use crate::query::insert_aliases_query::InsertAliasesQuery;
use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::insert_values_query::InsertValuesQuery;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryValues;
use crate::DbKeyValue;

pub struct Insert {}

impl Insert {
    pub fn alias(self, name: &str) -> InsertAlias {
        InsertAlias(InsertAliasesQuery {
            ids: QueryIds::Id(0.into()),
            aliases: vec![name.to_string()],
        })
    }

    pub fn aliases(self, names: &[String]) -> InsertAliases {
        InsertAliases(InsertAliasesQuery {
            ids: QueryIds::Ids(vec![]),
            aliases: names.to_vec(),
        })
    }

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

    pub fn values(self, key_values: &[DbKeyValue]) -> InsertValues {
        InsertValues(InsertValuesQuery {
            ids: QueryIds::Id(0.into()),
            values: QueryValues::Single(key_values.to_vec()),
        })
    }

    pub fn values_multi(self, key_values: &[&[DbKeyValue]]) -> InsertValuesMulti {
        InsertValuesMulti(InsertValuesQuery {
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Multi(key_values.iter().map(|v| v.to_vec()).collect()),
        })
    }
}
