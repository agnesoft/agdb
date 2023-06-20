use super::remove_aliases::RemoveAliases;
use super::remove_ids::RemoveIds;
use super::remove_values::RemoveValues;
use crate::query::query_aliases::QueryAliases;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryKeys;
use crate::query::remove_aliases_query::RemoveAliasesQuery;
use crate::query::remove_query::RemoveQuery;
use crate::query::remove_values_query::RemoveValuesQuery;
use crate::query::search_query::SearchQuery;
use crate::query::select_values_query::SelectValuesQuery;

pub struct Remove {}

impl Remove {
    pub fn aliases<T: Into<QueryAliases>>(self, names: T) -> RemoveAliases {
        RemoveAliases(RemoveAliasesQuery {
            aliases: Into::<QueryAliases>::into(names).0,
        })
    }

    pub fn ids<T: Into<QueryIds>>(self, ids: T) -> RemoveIds {
        RemoveIds(RemoveQuery(ids.into()))
    }

    pub fn search(self, query: SearchQuery) -> RemoveIds {
        RemoveIds(RemoveQuery(QueryIds::Search(query)))
    }

    pub fn values<T: Into<QueryKeys>>(self, keys: T) -> RemoveValues {
        RemoveValues(RemoveValuesQuery(SelectValuesQuery {
            keys: Into::<QueryKeys>::into(keys).0,
            ids: QueryIds::Ids(vec![0.into()]),
        }))
    }
}
