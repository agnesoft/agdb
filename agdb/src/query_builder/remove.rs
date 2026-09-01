use crate::Amend;
use crate::DbValue;
use crate::InsertValuesQuery;
use crate::QueryIds;
use crate::RemoveAliasesQuery;
use crate::RemoveQuery;
use crate::RemoveValuesQuery;
use crate::SearchQuery;
use crate::SelectValuesQuery;
use crate::db::db_value::DbValues;
use crate::query::query_aliases::QueryAliases;
use crate::query::query_values::MultiValues;
use crate::query::query_values::QueryValues;
use crate::query::query_values::SingleValues;
use crate::query_builder::insert_values::InsertValues;
use crate::query_builder::remove_aliases::RemoveAliases;
use crate::query_builder::remove_ids::RemoveIds;
use crate::query_builder::remove_index::RemoveIndex;
use crate::query_builder::remove_values::RemoveValues;
use crate::query_builder::search::Search;

/// Remove builder to choose what to delete from the database.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[cfg_attr(feature = "api", type_def(inherent))]
pub struct Remove {}

#[cfg_attr(feature = "api", agdb::impl_def())]
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

    /// Remove the elements found using the search query.
    /// Equivalent to `ids(QueryIds::Search(search)/*...*/)`.
    pub fn search(self) -> Search<RemoveQuery> {
        Search(RemoveQuery(QueryIds::Search(SearchQuery::new())))
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

    /// Amend-remove (decrement / remove-from) list of lists `key_values`
    /// on existing elements. For numerics this subtracts, for strings removes
    /// all occurrences, for vec types removes first occurrence of each element.
    /// If a key does not exist on the element, it is silently skipped.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::remove().amend([[("counter", 5).into()]]).ids(1);
    /// QueryBuilder::remove().amend([[("tags", vec!["old"]).into()]]).ids(1);
    /// ```
    pub fn amend<T: Into<MultiValues>>(self, key_values: T) -> InsertValues {
        InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Multi(Into::<MultiValues>::into(key_values).0),
            amend: Amend::Remove,
        })
    }

    /// Amend-remove uniformly: applies the same remove operation
    /// to all target elements.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::remove().amend_uniform([("counter", 5).into()]).ids(1);
    /// QueryBuilder::remove().amend_uniform([("counter", 5).into()]).ids([1, 2]);
    /// ```
    pub fn amend_uniform<T: Into<SingleValues>>(self, key_values: T) -> InsertValues {
        InsertValues(InsertValuesQuery {
            ids: QueryIds::Ids(vec![]),
            values: QueryValues::Single(Into::<SingleValues>::into(key_values).0),
            amend: Amend::Remove,
        })
    }
}
