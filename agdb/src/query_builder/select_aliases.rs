use crate::query_builder::search::Search;
use crate::QueryIds;
use crate::SearchQuery;
use crate::SelectAliasesQuery;
use crate::SelectAllAliasesQuery;

/// Select aliases builder.
pub struct SelectAliases(pub SelectAliasesQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct SelectAliasesIds(pub SelectAliasesQuery);

impl SelectAliases {
    /// An id or list of ids or search query to select aliases of.
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectAliasesIds {
        self.0 .0 = ids.into();

        SelectAliasesIds(self.0)
    }

    /// Select using the result of a search query as ids.
    /// Equivalent to `ids(QueryBuilder::search()/* ... */)`.
    pub fn search(mut self) -> Search<SelectAliasesQuery> {
        self.0 .0 = QueryIds::Search(SearchQuery::new());
        Search(self.0)
    }

    /// Returns the built `SelectAllAliases` object.
    pub fn query(self) -> SelectAllAliasesQuery {
        SelectAllAliasesQuery {}
    }
}

impl SelectAliasesIds {
    /// Returns the built `SelectAllAliases` object.
    pub fn query(self) -> SelectAliasesQuery {
        self.0
    }
}
