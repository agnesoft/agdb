use crate::QueryIds;
use crate::SearchQuery;
use crate::SelectAliasesQuery;
use crate::SelectAllAliasesQuery;
use crate::query_builder::search::Search;

/// Select aliases builder.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct SelectAliases(pub SelectAliasesQuery);

/// Final builder that lets you create
/// an actual query object.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct SelectAliasesIds(pub SelectAliasesQuery);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SelectAliases {
    /// An id or list of ids or search query to select aliases of.
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectAliasesIds {
        self.0.0 = ids.into();

        SelectAliasesIds(self.0)
    }

    /// Select using the result of a search query as ids.
    /// Equivalent to `ids(QueryBuilder::search()/* ... */)`.
    pub fn search(mut self) -> Search<SelectAliasesQuery> {
        self.0.0 = QueryIds::Search(SearchQuery::new());
        Search(self.0)
    }

    /// Returns the built `SelectAllAliases` object.
    pub fn query(self) -> SelectAllAliasesQuery {
        SelectAllAliasesQuery {}
    }
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SelectAliasesIds {
    /// Returns the built `SelectAllAliases` object.
    pub fn query(self) -> SelectAliasesQuery {
        self.0
    }
}
