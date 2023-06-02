use super::insert_aliases::InsertAliases;
use super::insert_edge::InsertEdges;
use super::insert_nodes::InsertNodes;
use super::insert_values::InsertValues;
use super::insert_values::InsertValuesUniform;
use crate::query::insert_aliases_query::InsertAliasesQuery;
use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::insert_values_query::InsertValuesQuery;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryValues;
use crate::DbKeyValue;

pub struct Insert {}

impl Insert {
    pub fn aliases(self, names: &[String]) -> InsertAliases {
        InsertAliases(InsertAliasesQuery {
            ids: QueryIds::Ids(vec![]),
            aliases: names.to_vec(),
        })
    }

    pub fn edges(self) -> InsertEdges {
        InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![]),
            to: QueryIds::Ids(vec![]),
            values: QueryValues::Single(vec![]),
            each: false,
        })
    }

    pub fn nodes(self) -> InsertNodes {
        InsertNodes(InsertNodesQuery {
            count: 0,
            values: QueryValues::Single(vec![]),
            aliases: vec![],
        })
    }

    pub fn values(self, key_values: &[&[DbKeyValue]]) -> InsertValues {
        InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Multi(key_values.iter().map(|v| v.to_vec()).collect()),
        })
    }

    pub fn values_uniform(self, key_values: &[DbKeyValue]) -> InsertValuesUniform {
        InsertValuesUniform(InsertValuesQuery {
            ids: QueryIds::Ids(vec![0.into()]),
            values: QueryValues::Single(key_values.to_vec()),
        })
    }
}
