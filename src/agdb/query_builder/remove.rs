use super::remove_aliases::RemoveAliases;
use super::remove_ids::RemoveIds;
use super::remove_values::RemoveValues;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::remove_aliases_query::RemoveAliasesQuery;
use crate::query::remove_query::RemoveQuery;
use crate::query::remove_values_query::RemoveValuesQuery;
use crate::query::search_query::SearchQuery;
use crate::query::select_values_query::SelectValuesQuery;
use crate::DbKey;

pub struct Remove {}

impl Remove {
    pub fn aliases(self, names: &[String]) -> RemoveAliases {
        RemoveAliases(RemoveAliasesQuery {
            aliases: names.to_vec(),
        })
    }

    pub fn ids(self, ids: &[QueryId]) -> RemoveIds {
        RemoveIds(RemoveQuery(QueryIds::Ids(ids.to_vec())))
    }

    pub fn search(self, query: SearchQuery) -> RemoveIds {
        RemoveIds(RemoveQuery(QueryIds::Search(query)))
    }

    pub fn values(self, keys: &[DbKey]) -> RemoveValues {
        RemoveValues(RemoveValuesQuery(SelectValuesQuery {
            keys: keys.to_vec(),
            ids: QueryIds::Ids(vec![0.into()]),
        }))
    }
}
