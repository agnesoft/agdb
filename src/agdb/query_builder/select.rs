use super::select_alias::SelectAlias;
use super::select_aliases::SelectAliases;
use super::select_ids::SelectIds;
use super::select_key_count::SelectKeyCount;
use super::select_keys::SelectKeys;
use super::select_values::SelectValues;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::query::select_aliases_query::SelectAliasesQuery;
use crate::query::select_key_count_query::SelectKeyCountQuery;
use crate::query::select_keys_query::SelectKeysQuery;
use crate::query::select_query::SelectQuery;
use crate::query::select_values_query::SelectValuesQuery;
use crate::DbKey;

pub struct Select {}

impl Select {
    pub fn alias(self) -> SelectAlias {
        SelectAlias(SelectAliasesQuery {
            ids: QueryIds::Ids(vec![0.into()]),
        })
    }

    pub fn aliases(self) -> SelectAliases {
        SelectAliases(SelectAliasesQuery {
            ids: QueryIds::Ids(vec![]),
        })
    }

    pub fn id(self, id: QueryId) -> SelectIds {
        SelectIds(SelectQuery(QueryIds::Ids(vec![id])))
    }

    pub fn ids(self, ids: &[QueryId]) -> SelectIds {
        SelectIds(SelectQuery(QueryIds::Ids(ids.to_vec())))
    }

    pub fn search(self, search: SearchQuery) -> SelectIds {
        SelectIds(SelectQuery(QueryIds::Search(search)))
    }

    pub fn keys(self) -> SelectKeys {
        SelectKeys(SelectKeysQuery(QueryIds::Ids(vec![0.into()])))
    }

    pub fn key_count(self) -> SelectKeyCount {
        SelectKeyCount(SelectKeyCountQuery(QueryIds::Ids(vec![0.into()])))
    }

    pub fn values(self, keys: &[DbKey]) -> SelectValues {
        SelectValues(SelectValuesQuery {
            keys: keys.to_vec(),
            ids: QueryIds::Ids(vec![0.into()]),
        })
    }
}
