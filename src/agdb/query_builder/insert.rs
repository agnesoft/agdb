use super::insert_aliases::InsertAliases;
use super::insert_edge::InsertEdges;
use super::insert_nodes::InsertNodes;
use super::insert_values::InsertValues;
use super::insert_values::InsertValuesUniform;
use crate::query::insert_aliases_query::InsertAliasesQuery;
use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::insert_values_query::InsertValuesQuery;
use crate::query::query_aliases::QueryAliases;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::MultiValues;
use crate::query::query_values::QueryValues;
use crate::query::query_values::SingleValues;

pub struct Insert {}

impl Insert {
    pub fn aliases<T: Into<QueryAliases>>(self, names: T) -> InsertAliases {
        InsertAliases(InsertAliasesQuery {
            ids: QueryIds::Ids(vec![]),
            aliases: Into::<QueryAliases>::into(names).0,
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

    pub fn values<T: Into<MultiValues>>(self, key_values: T) -> InsertValues {
        InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Multi(Into::<MultiValues>::into(key_values).0),
        })
    }

    pub fn values_uniform<T: Into<SingleValues>>(self, key_values: T) -> InsertValuesUniform {
        InsertValuesUniform(InsertValuesQuery {
            ids: QueryIds::Ids(vec![0.into()]),
            values: QueryValues::Single(Into::<SingleValues>::into(key_values).0),
        })
    }
}
