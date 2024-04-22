use crate::query::query_values::QueryKeys;
use crate::query_builder::select_aliases::SelectAliases;
use crate::query_builder::select_edge_count::SelectEdgeCount;
use crate::query_builder::select_ids::SelectIds;
use crate::query_builder::select_indexes::SelectIndexes;
use crate::query_builder::select_key_count::SelectKeyCount;
use crate::query_builder::select_keys::SelectKeys;
use crate::query_builder::select_values::SelectValues;
use crate::QueryIds;
use crate::SelectAliasesQuery;
use crate::SelectEdgeCountQuery;
use crate::SelectKeyCountQuery;
use crate::SelectKeysQuery;
use crate::SelectQuery;
use crate::SelectValuesQuery;

/// Select builder that lets you choose what
/// data you want to select form the database.
pub struct Select {}

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
            ids: QueryIds::Ids(vec![0.into()]),
            from: true,
            to: true,
        })
    }

    /// Select number of outgoing edges. Each
    /// element of the result withll have a proeprty `String("edge_count")`
    /// with u64 as the value.
    pub fn edge_count_from(self) -> SelectEdgeCount {
        SelectEdgeCount(SelectEdgeCountQuery {
            ids: QueryIds::Ids(vec![0.into()]),
            from: true,
            to: false,
        })
    }

    /// Select number of incoming edges. Each
    /// element of the result withll have a proeprty `String("edge_count")`
    /// with u64 as the value.
    pub fn edge_count_to(self) -> SelectEdgeCount {
        SelectEdgeCount(SelectEdgeCountQuery {
            ids: QueryIds::Ids(vec![0.into()]),
            from: false,
            to: true,
        })
    }

    /// Select elements with `ids` with all properties (key-values).
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(self, ids: T) -> SelectIds {
        SelectIds(SelectQuery(ids.into()))
    }

    /// Select all indexes in the database. The returned result
    /// will contain single element with each value representing
    /// index name and number of indexed values in that index.
    pub fn indexes(self) -> SelectIndexes {
        SelectIndexes {}
    }

    /// Select keys only (values will be empty).
    pub fn keys(self) -> SelectKeys {
        SelectKeys(SelectKeysQuery(QueryIds::Ids(vec![0.into()])))
    }

    /// Select number of keys. Each element of the result will have
    /// a property `String("key_count")` with `u64` as the value.
    pub fn key_count(self) -> SelectKeyCount {
        SelectKeyCount(SelectKeyCountQuery(QueryIds::Ids(vec![0.into()])))
    }

    /// Select elements with `ids` with only `keys` properties (key-values).
    /// All ids specified must exist in the database.
    pub fn values<T: Into<QueryKeys>>(self, keys: T) -> SelectValues {
        SelectValues(SelectValuesQuery {
            keys: Into::<QueryKeys>::into(keys).0,
            ids: QueryIds::Ids(vec![0.into()]),
        })
    }
}
