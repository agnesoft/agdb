use super::select_aliases::SelectAliases;
use super::select_ids::SelectIds;
use super::select_key_count::SelectKeyCount;
use super::select_keys::SelectKeys;
use super::select_values::SelectValues;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryKeys;
use crate::query::select_aliases_query::SelectAliasesQuery;
use crate::query::select_key_count_query::SelectKeyCountQuery;
use crate::query::select_keys_query::SelectKeysQuery;
use crate::query::select_query::SelectQuery;
use crate::query::select_values_query::SelectValuesQuery;
use crate::DbUserValue;

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

    /// Select elements with `ids` with only `keys` properties (key-values)
    /// that constitute the user type `T`. All ids specified must exist in
    /// the database.
    pub fn values_t<T: DbUserValue>(self) -> SelectValues {
        SelectValues(SelectValuesQuery {
            keys: T::db_keys(),
            ids: QueryIds::Ids(vec![0.into()]),
        })
    }
}
