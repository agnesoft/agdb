use crate::query::query_values::QueryKeys;
use crate::query_builder::select_aliases::SelectAliases;
use crate::query_builder::select_ids::SelectIds;
use crate::query_builder::select_key_count::SelectKeyCount;
use crate::query_builder::select_keys::SelectKeys;
use crate::query_builder::select_values::SelectValues;
use crate::QueryIds;
use crate::SelectAliasesQuery;
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

    /// Select elements with `ids` with all properties (key-values).
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(self, ids: T) -> SelectIds {
        SelectIds(SelectQuery(ids.into()))
    }

    /// Select keys only (values will be empty).
    pub fn keys(self) -> SelectKeys {
        SelectKeys(SelectKeysQuery(QueryIds::Ids(vec![0.into()])))
    }

    /// Select number of keys. Each element of the result will have
    /// a property `String("key_count")` with `i64` as the value.
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
