use super::select_aliases::SelectAliases;
use super::select_ids::SelectIds;
use super::select_key_count::SelectKeyCount;
use super::select_keys::SelectKeys;
use super::select_values::SelectValues;
use crate::query::query_ids::QueryIds;
use crate::query::query_values::QueryKeys;
use crate::query::search_query::SearchQuery;
use crate::query::select_aliases_query::SelectAliasesQuery;
use crate::query::select_key_count_query::SelectKeyCountQuery;
use crate::query::select_keys_query::SelectKeysQuery;
use crate::query::select_query::SelectQuery;
use crate::query::select_values_query::SelectValuesQuery;

pub struct Select {}

impl Select {
    pub fn aliases(self) -> SelectAliases {
        SelectAliases(SelectAliasesQuery(QueryIds::Ids(vec![])))
    }

    pub fn ids<T: Into<QueryIds>>(self, ids: T) -> SelectIds {
        SelectIds(SelectQuery(ids.into()))
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

    pub fn values<T: Into<QueryKeys>>(self, keys: T) -> SelectValues {
        SelectValues(SelectValuesQuery {
            keys: Into::<QueryKeys>::into(keys).0,
            ids: QueryIds::Ids(vec![0.into()]),
        })
    }
}
