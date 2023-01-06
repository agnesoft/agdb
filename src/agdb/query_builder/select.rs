use super::search::Search;
use super::select_alias::SelectAlias;
use super::select_aliases::SelectAliases;
use super::select_id::SelectId;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::query::select_aliases_query::SelectAliasesQuery;

pub struct Select {}

impl Select {
    pub fn alias(self) -> SelectAlias {
        SelectAlias(SelectAliasesQuery {
            ids: QueryIds::Id(0.into()),
        })
    }

    pub fn aliases(self) -> SelectAliases {
        SelectAliases(SelectAliasesQuery {
            ids: QueryIds::Ids(vec![]),
        })
    }

    pub fn count(self) -> SelectId {
        SelectId(SearchQuery {
            origin: QueryId::Id(0),
            destination: QueryId::Id(0),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
    }

    pub fn id(self) -> SelectId {
        SelectId(SearchQuery {
            origin: QueryId::Id(0),
            destination: QueryId::Id(0),
            limit: 1,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
    }

    pub fn ids(self) -> Search {
        Search(SearchQuery {
            origin: QueryId::Id(0),
            destination: QueryId::Id(0),
            limit: 1,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
    }
}
