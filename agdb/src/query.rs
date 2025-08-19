pub mod insert_aliases_query;
pub mod insert_edges_query;
pub mod insert_index_query;
pub mod insert_nodes_query;
pub mod insert_values_query;
pub mod query_aliases;
pub mod query_condition;
pub mod query_id;
pub mod query_ids;
pub mod query_result;
pub mod query_values;
pub mod remove_aliases_query;
pub mod remove_index_query;
pub mod remove_query;
pub mod remove_values_query;
pub mod search_query;
pub mod select_aliases_query;
pub mod select_all_aliases_query;
pub mod select_edge_count_query;
pub mod select_indexes_query;
pub mod select_key_count_query;
pub mod select_keys_query;
pub mod select_node_count;
pub mod select_values_query;

use crate::DbError;
use crate::DbImpl;
use crate::QueryResult;
use crate::StorageData;

/// Trait for immutable `agdb` database queries. This
/// trait is unlikely to be implementable for user types.
pub trait Query {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError>;
}

/// Trait for mutable `agdb` database queries. This
/// trait is unlikely to be implementable for user types.
pub trait QueryMut {
    fn process<Store: StorageData>(&self, db: &mut DbImpl<Store>) -> Result<QueryResult, DbError>;
}

#[cfg(any(feature = "serde", feature = "openapi"))]
use crate::{
    InsertAliasesQuery, InsertEdgesQuery, InsertIndexQuery, InsertNodesQuery, InsertValuesQuery,
    RemoveAliasesQuery, RemoveIndexQuery, RemoveQuery, RemoveValuesQuery, SearchQuery,
    SelectAliasesQuery, SelectAllAliasesQuery, SelectEdgeCountQuery, SelectIndexesQuery,
    SelectKeyCountQuery, SelectKeysQuery, SelectNodeCountQuery, SelectValuesQuery,
};

/// Convenience enum for serializing/deserializing queries.
#[cfg(any(feature = "serde", feature = "openapi"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
#[derive(Clone, Debug, PartialEq)]
#[expect(clippy::large_enum_variant)]
pub enum QueryType {
    InsertAlias(InsertAliasesQuery),
    InsertEdges(InsertEdgesQuery),
    InsertIndex(InsertIndexQuery),
    InsertNodes(InsertNodesQuery),
    InsertValues(InsertValuesQuery),
    Remove(RemoveQuery),
    RemoveAliases(RemoveAliasesQuery),
    RemoveIndex(RemoveIndexQuery),
    RemoveValues(RemoveValuesQuery),
    Search(SearchQuery),
    SelectAliases(SelectAliasesQuery),
    SelectAllAliases(SelectAllAliasesQuery),
    SelectEdgeCount(SelectEdgeCountQuery),
    SelectIndexes(SelectIndexesQuery),
    SelectKeys(SelectKeysQuery),
    SelectKeyCount(SelectKeyCountQuery),
    SelectNodeCount(SelectNodeCountQuery),
    SelectValues(SelectValuesQuery),
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<InsertAliasesQuery> for QueryType {
    fn from(value: InsertAliasesQuery) -> Self {
        QueryType::InsertAlias(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<InsertIndexQuery> for QueryType {
    fn from(value: InsertIndexQuery) -> Self {
        QueryType::InsertIndex(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<InsertEdgesQuery> for QueryType {
    fn from(value: InsertEdgesQuery) -> Self {
        QueryType::InsertEdges(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<InsertNodesQuery> for QueryType {
    fn from(value: InsertNodesQuery) -> Self {
        QueryType::InsertNodes(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<InsertValuesQuery> for QueryType {
    fn from(value: InsertValuesQuery) -> Self {
        QueryType::InsertValues(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<RemoveQuery> for QueryType {
    fn from(value: RemoveQuery) -> Self {
        QueryType::Remove(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<RemoveAliasesQuery> for QueryType {
    fn from(value: RemoveAliasesQuery) -> Self {
        QueryType::RemoveAliases(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<RemoveIndexQuery> for QueryType {
    fn from(value: RemoveIndexQuery) -> Self {
        QueryType::RemoveIndex(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<RemoveValuesQuery> for QueryType {
    fn from(value: RemoveValuesQuery) -> Self {
        QueryType::RemoveValues(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<SearchQuery> for QueryType {
    fn from(value: SearchQuery) -> Self {
        QueryType::Search(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<SelectAliasesQuery> for QueryType {
    fn from(value: SelectAliasesQuery) -> Self {
        QueryType::SelectAliases(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<SelectAllAliasesQuery> for QueryType {
    fn from(value: SelectAllAliasesQuery) -> Self {
        QueryType::SelectAllAliases(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<SelectEdgeCountQuery> for QueryType {
    fn from(value: SelectEdgeCountQuery) -> Self {
        QueryType::SelectEdgeCount(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<SelectIndexesQuery> for QueryType {
    fn from(value: SelectIndexesQuery) -> Self {
        QueryType::SelectIndexes(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<SelectKeyCountQuery> for QueryType {
    fn from(value: SelectKeyCountQuery) -> Self {
        QueryType::SelectKeyCount(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<SelectKeysQuery> for QueryType {
    fn from(value: SelectKeysQuery) -> Self {
        QueryType::SelectKeys(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<SelectNodeCountQuery> for QueryType {
    fn from(value: SelectNodeCountQuery) -> Self {
        QueryType::SelectNodeCount(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
impl From<SelectValuesQuery> for QueryType {
    fn from(value: SelectValuesQuery) -> Self {
        QueryType::SelectValues(value)
    }
}

#[cfg(any(feature = "serde", feature = "openapi"))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::AgdbSerialize;
    use crate::DbKeyOrder;
    use crate::QueryBuilder;

    #[test]
    fn derives() {
        let queries: Vec<QueryType> = vec![
            QueryBuilder::insert().nodes().count(2).query().into(),
            QueryBuilder::insert()
                .aliases(["node1", "node2"])
                .ids([1, 2])
                .query()
                .into(),
            QueryBuilder::insert()
                .edges()
                .from("node1")
                .to("node2")
                .query()
                .into(),
            QueryBuilder::insert()
                .values([[("key", 1.1).into()]])
                .ids("node1")
                .query()
                .into(),
            QueryBuilder::insert().index("key").query().into(),
            QueryBuilder::search().from(1).query().into(),
            QueryBuilder::select().ids(1).query().into(),
            QueryBuilder::select().aliases().ids(1).query().into(),
            QueryBuilder::select().aliases().query().into(),
            QueryBuilder::select().indexes().query().into(),
            QueryBuilder::select().keys().ids(1).query().into(),
            QueryBuilder::select().key_count().ids(1).query().into(),
            QueryBuilder::select().edge_count().ids(1).query().into(),
            QueryBuilder::select().node_count().query().into(),
            QueryBuilder::select().values("key").ids(1).query().into(),
            QueryBuilder::remove().aliases("node2").query().into(),
            QueryBuilder::remove().index("key").query().into(),
            QueryBuilder::remove().values("key").ids(1).query().into(),
            QueryBuilder::remove().ids("node1").query().into(),
            QueryBuilder::search()
                .depth_first()
                .to(1)
                .order_by(DbKeyOrder::Asc("id".into()))
                .offset(10)
                .limit(10)
                .where_()
                .node()
                .or()
                .edge()
                .and()
                .not_beyond()
                .keys("hidden")
                .and()
                .where_()
                .distance(crate::CountComparison::Equal(2))
                .or()
                .key("key")
                .value(crate::Comparison::NotEqual("value".into()))
                .query()
                .into(),
        ];
        let _ = format!("{queries:?}");
        assert_eq!(queries, queries);

        let serialized = AgdbSerialize::serialize(&queries);
        let deserialized: Vec<QueryType> = AgdbSerialize::deserialize(&serialized).unwrap();
        assert_eq!(queries, deserialized);
    }

    #[test]
    fn serialization() {
        let query: QueryType = QueryBuilder::insert().nodes().count(2).query().into();
        let serialized = AgdbSerialize::serialize(&query);
        let deserialized: QueryType = AgdbSerialize::deserialize(&serialized).unwrap();
        assert_eq!(query, deserialized);
    }
}
