use super::remove_aliases::RemoveAliases;
use super::remove_ids::RemoveIds;
use super::remove_values::RemoveValues;
use crate::query::query_aliases::QueryAliases;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryKeys;
use crate::query::remove_aliases_query::RemoveAliasesQuery;
use crate::query::remove_query::RemoveQuery;
use crate::query::remove_values_query::RemoveValuesQuery;
use crate::query::select_values_query::SelectValuesQuery;

/// Remove builder to choose what to delete from the database.
pub struct Remove {}

impl Remove {
    /// List of aliases to delete from the database. It is not an error
    /// if any of the aliases does not exist in the database.
    pub fn aliases<T: Into<QueryAliases>>(self, names: T) -> RemoveAliases {
        RemoveAliases(RemoveAliasesQuery(Into::<QueryAliases>::into(names).0))
    }

    /// Id, list of ids or search of the database elements to delete
    /// from the database.
    ///
    /// NOTE: all properties (key-value pairs) associated
    /// with the elements will be also deleted. If deleting nodes its outgoing
    /// and incoming edges will also be deleted along with their properties.
    ///
    /// It is not an error if not all of the ids exist in the database.
    pub fn ids<T: Into<QueryIds>>(self, ids: T) -> RemoveIds {
        RemoveIds(RemoveQuery(ids.into()))
    }

    /// List of keys to delete from ids selected in the next step. It is not an
    /// error if not all of the keys exist on the elements.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::remove().values(vec!["k".into()]).ids(1);
    /// QueryBuilder::remove().values(vec!["k".into()]).ids(vec![1]);
    /// QueryBuilder::remove().values(vec!["k".into()]).ids(QueryBuilder::search().from(1).query());
    /// ```
    pub fn values<T: Into<QueryKeys>>(self, keys: T) -> RemoveValues {
        RemoveValues(RemoveValuesQuery(SelectValuesQuery {
            keys: Into::<QueryKeys>::into(keys).0,
            ids: QueryIds::Ids(vec![0.into()]),
        }))
    }
}
