use super::remove_alias::RemoveAlias;
use super::remove_ids::RemoveIds;
use super::remove_values::RemoveValues;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::remove_aliases_query::RemoveAliasesQuery;
use crate::query::remove_query::RemoveQuery;
use crate::query::remove_values_query::RemoveValuesQuery;
use crate::query::search_query::SearchQuery;
use crate::query::select_query::SelectQuery;
use crate::DbKey;

pub struct Remove {}

impl Remove {
    pub fn alias(self, name: &str) -> RemoveAlias {
        RemoveAlias(RemoveAliasesQuery {
            aliases: vec![name.to_string()],
        })
    }

    pub fn aliases(self, names: &[String]) -> RemoveAlias {
        RemoveAlias(RemoveAliasesQuery {
            aliases: names.to_vec(),
        })
    }

    pub fn id(self, id: QueryId) -> RemoveIds {
        RemoveIds(RemoveQuery(QueryIds::Id(id)))
    }

    pub fn ids(self, ids: &[QueryId]) -> RemoveIds {
        RemoveIds(RemoveQuery(QueryIds::Ids(ids.to_vec())))
    }

    pub fn query(self, query: SearchQuery) -> RemoveIds {
        RemoveIds(RemoveQuery(QueryIds::Search(query)))
    }

    pub fn value(self, key: DbKey) -> RemoveValues {
        RemoveValues(RemoveValuesQuery(SelectQuery {
            keys: vec![key],
            ids: QueryIds::Id(0.into()),
        }))
    }

    pub fn values(self, keys: &[DbKey]) -> RemoveValues {
        RemoveValues(RemoveValuesQuery(SelectQuery {
            keys: keys.to_vec(),
            ids: QueryIds::Id(0.into()),
        }))
    }
}
