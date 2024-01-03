pub mod insert_aliases_query;
pub mod insert_edges_query;
pub mod insert_index_query;
pub mod insert_nodes_query;
pub mod insert_values_query;
pub mod query_aliases;
pub mod query_condition;
pub mod query_error;
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
pub mod select_indexes_query;
pub mod select_key_count_query;
pub mod select_keys_query;
pub mod select_query;
pub mod select_values_query;

use crate::DbImpl;
use crate::QueryError;
use crate::QueryResult;
use crate::StorageData;

/// Trait for immutable `agdb` database queries. This
/// trait is unlikely to be implementable for user types.
pub trait Query {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError>;
}

/// Trait for mutable `agdb` database queries. This
/// trait is unlikely to be implementable for user types.
pub trait QueryMut {
    fn process<Store: StorageData>(
        &self,
        db: &mut DbImpl<Store>,
    ) -> Result<QueryResult, QueryError>;
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
use crate::{
    InsertAliasesQuery, InsertEdgesQuery, InsertIndexQuery, InsertNodesQuery, InsertValuesQuery,
    RemoveAliasesQuery, RemoveIndexQuery, RemoveQuery, RemoveValuesQuery, SearchQuery,
    SelectAliasesQuery, SelectAllAliasesQuery, SelectIndexesQuery, SelectKeyCountQuery,
    SelectKeysQuery, SelectQuery, SelectValuesQuery,
};

/// Convenience enum for serializing/deserializing queries.
#[cfg(any(feature = "serde", feature = "opeanapi"))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
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
    Select(SelectQuery),
    SelectAliases(SelectAliasesQuery),
    SelectAllAliases(SelectAllAliasesQuery),
    SelectIndexes(SelectIndexesQuery),
    SelectKeys(SelectKeysQuery),
    SelectKeyCount(SelectKeyCountQuery),
    SelectValues(SelectValuesQuery),
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<InsertAliasesQuery> for QueryType {
    fn from(value: InsertAliasesQuery) -> Self {
        QueryType::InsertAlias(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<InsertIndexQuery> for QueryType {
    fn from(value: InsertIndexQuery) -> Self {
        QueryType::InsertIndex(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<InsertEdgesQuery> for QueryType {
    fn from(value: InsertEdgesQuery) -> Self {
        QueryType::InsertEdges(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<InsertNodesQuery> for QueryType {
    fn from(value: InsertNodesQuery) -> Self {
        QueryType::InsertNodes(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<InsertValuesQuery> for QueryType {
    fn from(value: InsertValuesQuery) -> Self {
        QueryType::InsertValues(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<RemoveQuery> for QueryType {
    fn from(value: RemoveQuery) -> Self {
        QueryType::Remove(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<RemoveAliasesQuery> for QueryType {
    fn from(value: RemoveAliasesQuery) -> Self {
        QueryType::RemoveAliases(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<RemoveIndexQuery> for QueryType {
    fn from(value: RemoveIndexQuery) -> Self {
        QueryType::RemoveIndex(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<RemoveValuesQuery> for QueryType {
    fn from(value: RemoveValuesQuery) -> Self {
        QueryType::RemoveValues(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<SearchQuery> for QueryType {
    fn from(value: SearchQuery) -> Self {
        QueryType::Search(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<SelectQuery> for QueryType {
    fn from(value: SelectQuery) -> Self {
        QueryType::Select(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<SelectAliasesQuery> for QueryType {
    fn from(value: SelectAliasesQuery) -> Self {
        QueryType::SelectAliases(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<SelectAllAliasesQuery> for QueryType {
    fn from(value: SelectAllAliasesQuery) -> Self {
        QueryType::SelectAllAliases(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<SelectIndexesQuery> for QueryType {
    fn from(value: SelectIndexesQuery) -> Self {
        QueryType::SelectIndexes(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<SelectKeyCountQuery> for QueryType {
    fn from(value: SelectKeyCountQuery) -> Self {
        QueryType::SelectKeyCount(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<SelectKeysQuery> for QueryType {
    fn from(value: SelectKeysQuery) -> Self {
        QueryType::SelectKeys(value)
    }
}

#[cfg(any(feature = "serde", feature = "opeanapi"))]
impl From<SelectValuesQuery> for QueryType {
    fn from(value: SelectValuesQuery) -> Self {
        QueryType::SelectValues(value)
    }
}
