use crate::DbUserValue;
use crate::QueryIds;
use crate::SelectAliasesQuery;
use crate::SelectEdgeCountQuery;
use crate::SelectKeyCountQuery;
use crate::SelectKeysQuery;
use crate::SelectValuesQuery;
use crate::db::db_value::DbValues;
use crate::query_builder::search::Search;
use crate::query_builder::select_aliases::SelectAliases;
use crate::query_builder::select_edge_count::SelectEdgeCount;
use crate::query_builder::select_ids::SelectIds;
use crate::query_builder::select_indexes::SelectIndexes;
use crate::query_builder::select_key_count::SelectKeyCount;
use crate::query_builder::select_keys::SelectKeys;
use crate::query_builder::select_node_count::SelectNodeCount;
use crate::query_builder::select_values::SelectValues;

/// Select builder that lets you choose what
/// data you want to select form the database.
#[cfg_attr(feature = "api", derive(agdb::ApiDefImpl))]
pub struct Select {}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl Select {
    /// Select aliases. If no ids are given all aliases
    /// in the database will be selected. Each element
    /// of the result will have a property `String("alias")`
    /// holding the alias. If `ids` are specified and any
    /// of them does not have an alias an error will occur
    /// when running such query.
    pub fn aliases(self) -> SelectAliases {
        SelectAliases(SelectAliasesQuery(QueryIds::Ids(vec![])))
    }

    /// Select number of outgoing and incoming edges. Each
    /// element of the result withll have a proeprty `String("edge_count")`
    /// with u64 as the value.
    pub fn edge_count(self) -> SelectEdgeCount {
        SelectEdgeCount(SelectEdgeCountQuery {
            ids: QueryIds::Ids(vec![]),
            from: true,
            to: true,
        })
    }

    /// Select number of outgoing edges. Each
    /// element of the result withll have a proeprty `String("edge_count")`
    /// with u64 as the value.
    pub fn edge_count_from(self) -> SelectEdgeCount {
        SelectEdgeCount(SelectEdgeCountQuery {
            ids: QueryIds::Ids(vec![]),
            from: true,
            to: false,
        })
    }

    /// Select number of incoming edges. Each
    /// element of the result withll have a proeprty `String("edge_count")`
    /// with u64 as the value.
    pub fn edge_count_to(self) -> SelectEdgeCount {
        SelectEdgeCount(SelectEdgeCountQuery {
            ids: QueryIds::Ids(vec![]),
            from: false,
            to: true,
        })
    }

    /// Select elements with `ids` with only `T::db_keys()`
    /// properties (key-values). All ids specified must
    /// exist in the database. Same as calling `values(T::db_keys())`.
    pub fn elements<T: DbUserValue>(self) -> SelectValues {
        SelectValues(SelectValuesQuery {
            keys: T::db_keys(),
            ids: QueryIds::Ids(vec![]),
        })
    }

    /// Select elements with `ids` with all properties (key-values).
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(self, ids: T) -> SelectIds {
        SelectIds(SelectValuesQuery {
            keys: vec![],
            ids: ids.into(),
        })
    }

    /// Select all indexes in the database. The returned result
    /// will contain single element with each value representing
    /// index name and number of indexed values in that index.
    pub fn indexes(self) -> SelectIndexes {
        SelectIndexes {}
    }

    /// Select keys only (values will be empty).
    pub fn keys(self) -> SelectKeys {
        SelectKeys(SelectKeysQuery(QueryIds::Ids(vec![])))
    }

    /// Select number of keys. Each element of the result will have
    /// a property `String("key_count")` with `u64` as the value.
    pub fn key_count(self) -> SelectKeyCount {
        SelectKeyCount(SelectKeyCountQuery(QueryIds::Ids(vec![])))
    }

    /// Select number of nodes in the database. The result will be a
    /// single element with a property `String("node_count")` with `u64`
    /// as the value.
    pub fn node_count(self) -> SelectNodeCount {
        SelectNodeCount {}
    }

    /// Select with all properties (key-values) using result
    /// of the search query as ids. Equivalent to `ids(QueryBuilder::search()/* ... */)`.
    pub fn search(self) -> Search<SelectValuesQuery> {
        Search(SelectValuesQuery {
            keys: vec![],
            ids: QueryIds::Search(crate::SearchQuery::new()),
        })
    }

    /// Select elements with `ids` with only `keys` properties (key-values).
    /// All ids specified must exist in the database.
    pub fn values<T: Into<DbValues>>(self, keys: T) -> SelectValues {
        SelectValues(SelectValuesQuery {
            keys: Into::<DbValues>::into(keys).0,
            ids: QueryIds::Ids(vec![]),
        })
    }
}
