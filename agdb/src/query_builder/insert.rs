use crate::DbUserValue;
use crate::DbValue;
use crate::InsertAliasesQuery;
use crate::InsertEdgesQuery;
use crate::InsertNodesQuery;
use crate::InsertValuesQuery;
use crate::QueryIds;
use crate::query::query_aliases::QueryAliases;
use crate::query::query_values::MultiValues;
use crate::query::query_values::QueryValues;
use crate::query::query_values::SingleValues;
use crate::query_builder::insert_aliases::InsertAliases;
use crate::query_builder::insert_edge::InsertEdges;
use crate::query_builder::insert_index::InsertIndex;
use crate::query_builder::insert_nodes::InsertNodes;
use crate::query_builder::insert_values::InsertValues;
use crate::query_builder::insert_values::InsertValuesIds;

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
    /// QueryBuilder::insert().aliases(["a", "b"]).ids([1, 2]);
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
    /// QueryBuilder::insert().edges().from([1, 2]);
    /// QueryBuilder::insert().edges().from(QueryBuilder::search().from(1).query());
    /// QueryBuilder::insert().edges().ids(-3);
    /// QueryBuilder::insert().edges().ids([-3, -4]);
    /// QueryBuilder::insert().edges().ids(QueryBuilder::search().from(1).where_().edge().query());
    /// ```
    pub fn edges(self) -> InsertEdges {
        InsertEdges(InsertEdgesQuery {
            from: QueryIds::Ids(vec![]),
            to: QueryIds::Ids(vec![]),
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Single(vec![]),
            each: false,
        })
    }

    /// Inserts `elem` into the database. The `elem`
    /// must implement (or derive) `DbUserValue` that will
    /// provide the `DbId` to be inserted to and conversion
    /// to the values. The ids must be `Some` and valid
    /// int the database.
    pub fn element<T: DbUserValue>(self, elem: &T) -> InsertValuesIds {
        InsertValuesIds(InsertValuesQuery {
            ids: QueryIds::Ids(vec![elem.db_id().unwrap_or_default()]),
            values: QueryValues::Multi(vec![elem.to_db_values()]),
        })
    }

    /// Inserts the `elems` into the database. Each `elem`
    /// must implement (or derive) `DbUserValue` that will
    /// provide the `DbId` to be inserted to and conversion
    /// to the values. The ids must be `Some` and valid
    /// int the database.
    pub fn elements<T: DbUserValue>(self, elems: &[T]) -> InsertValuesIds {
        let mut ids = vec![];
        let mut values = vec![];
        ids.reserve(elems.len());
        values.reserve(elems.len());

        elems.iter().for_each(|v| {
            ids.push(v.db_id().unwrap_or_default());
            values.push(v.to_db_values());
        });

        InsertValuesIds(InsertValuesQuery {
            ids: QueryIds::Ids(ids),
            values: QueryValues::Multi(values),
        })
    }

    /// Key to index on all elements in the database.
    pub fn index<T: Into<DbValue>>(self, key: T) -> InsertIndex {
        InsertIndex(key.into())
    }

    /// Inserts nodes into the database:
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().nodes().count(1);
    /// QueryBuilder::insert().nodes().aliases("a");
    /// QueryBuilder::insert().nodes().aliases(["a", "b"]);
    /// QueryBuilder::insert().nodes().ids(1);
    /// QueryBuilder::insert().nodes().ids([1, 2]);
    /// QueryBuilder::insert().nodes().ids("a");
    /// QueryBuilder::insert().nodes().ids(["a", "b"]);
    /// QueryBuilder::insert().nodes().ids(QueryBuilder::search().from(1).query());
    /// QueryBuilder::insert().nodes().values([[("k", 1).into()]]);
    /// ```
    pub fn nodes(self) -> InsertNodes {
        InsertNodes(InsertNodesQuery {
            count: 0,
            values: QueryValues::Single(vec![]),
            aliases: vec![],
            ids: QueryIds::Ids(vec![]),
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
    /// QueryBuilder::insert().values([[("k", 1).into()]]).ids(1);
    /// QueryBuilder::insert().values([[("k", 1).into()], [("k", 2).into()]]).ids([1, 2]);
    /// QueryBuilder::insert().values([[("k", 1).into()]]).ids(QueryBuilder::search().from(1).query());
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
    /// QueryBuilder::insert().values_uniform([("k", 1).into()]).ids(1);
    /// QueryBuilder::insert().values_uniform([("k", 1).into()]).ids([1, 2]);
    /// QueryBuilder::insert().values_uniform([("k", 1).into()]).ids(QueryBuilder::search().from(1).query());
    /// ```
    pub fn values_uniform<T: Into<SingleValues>>(self, key_values: T) -> InsertValues {
        InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Single(Into::<SingleValues>::into(key_values).0),
        })
    }
}
