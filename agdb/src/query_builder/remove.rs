use crate::db::db_value::DbValues;
use crate::query::query_aliases::QueryAliases;
use crate::query_builder::remove_aliases::RemoveAliases;
use crate::query_builder::remove_ids::RemoveIds;
use crate::query_builder::remove_index::RemoveIndex;
use crate::query_builder::remove_values::RemoveValues;
use crate::DbValue;
use crate::QueryIds;
use crate::RemoveAliasesQuery;
use crate::RemoveQuery;
use crate::RemoveValuesQuery;
use crate::SelectValuesQuery;

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

    /// Index to be removed from the database.
    pub fn index<T: Into<DbValue>>(self, key: T) -> RemoveIndex {
        RemoveIndex(key.into())
    }

    /// List of keys to delete from ids selected in the next step. It is not an
    /// error if not all of the keys exist on the elements.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::remove().values("k").ids(1);
    /// QueryBuilder::remove().values("k").ids([1, 2]);
    /// QueryBuilder::remove().values("k").ids(QueryBuilder::search().from(1).query());
    /// ```
    pub fn values<T: Into<DbValues>>(self, keys: T) -> RemoveValues {
        RemoveValues(RemoveValuesQuery(SelectValuesQuery {
            keys: Into::<DbValues>::into(keys).0,
            ids: QueryIds::Ids(vec![]),
        }))
    }
}
