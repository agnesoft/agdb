use super::insert_aliases::InsertAliases;
use super::insert_edge::InsertEdges;
use super::insert_nodes::InsertNodes;
use super::insert_values::InsertValues;
use crate::query::insert_aliases_query::InsertAliasesQuery;
use crate::query::insert_edges_query::InsertEdgesQuery;
use crate::query::insert_nodes_query::InsertNodesQuery;
use crate::query::insert_values_query::InsertValuesQuery;
use crate::query::query_aliases::QueryAliases;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::MultiValues;
use crate::query::query_values::QueryValues;
use crate::query::query_values::SingleValues;

/// Insert builder for inserting various data
/// into the database.
pub struct Insert {}

impl Insert {
    /// Inserts aliases `names` into the database:
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().aliases("a").ids(1);
    /// QueryBuilder::insert().aliases(vec!["a", "b"]).ids(vec![1, 2]);
    /// ```
    pub fn aliases<T: Into<QueryAliases>>(self, names: T) -> InsertAliases {
        InsertAliases(InsertAliasesQuery {
            ids: QueryIds::Ids(vec![]),
            aliases: Into::<QueryAliases>::into(names).0,
        })
    }

    /// Inserts edges into the database:
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().edges().from(1);
    /// QueryBuilder::insert().edges().from(vec![1, 2]);
    /// QueryBuilder::insert().edges().from(QueryBuilder::search().from(1).query());
    /// ```
    pub fn edges(self) -> InsertEdges {
        InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![]),
            to: QueryIds::Ids(vec![]),
            values: QueryValues::Single(vec![]),
            each: false,
        })
    }

    /// Inserts nodes into the database:
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().nodes();
    /// ```
    pub fn nodes(self) -> InsertNodes {
        InsertNodes(InsertNodesQuery {
            count: 0,
            values: QueryValues::Single(vec![]),
            aliases: vec![],
        })
    }

    /// Inserts or updates list of lists `key_values` into the database.
    /// Each item in the list represents a list of key-value pairs to
    /// be inserted into database elements identified by ids in the next step.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().values(vec![vec![("k", 1).into()]]).ids(1);
    /// QueryBuilder::insert().values(vec![vec![("k", 1).into()], vec![("k", 2).into()]]).ids(vec![1, 2]);
    /// QueryBuilder::insert().values(vec![vec![("k", 1).into()]]).ids(QueryBuilder::search().from(1).query());
    /// ```
    pub fn values<T: Into<MultiValues>>(self, key_values: T) -> InsertValues {
        InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Multi(Into::<MultiValues>::into(key_values).0),
        })
    }

    /// Inserts or updates list of `key_values` into the database.
    /// The list represents a list of key-value pairs to be inserted
    /// into every database elements identified by ids in the next step.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().values_uniform(vec![("k", 1).into()]).ids(1);
    /// QueryBuilder::insert().values_uniform(vec![("k", 1).into()]).ids(vec![1, 2]);
    /// QueryBuilder::insert().values_uniform(vec![("k", 1).into()]).ids(QueryBuilder::search().from(1).query());
    /// ```
    pub fn values_uniform<T: Into<SingleValues>>(self, key_values: T) -> InsertValues {
        InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![0.into()]),
            values: QueryValues::Single(Into::<SingleValues>::into(key_values).0),
        })
    }
}
