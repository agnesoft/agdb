use crate::InsertAliasesQuery;
use crate::QueryIds;
use crate::SearchQuery;
use crate::query_builder::search::Search;

/// Insert aliases builder to select `ids`
/// of the aliases.
pub struct InsertAliases(pub InsertAliasesQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct InsertAliasesIds(pub InsertAliasesQuery);

impl InsertAliases {
    /// An id or list of ids or search query to which to assign the aliases.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> InsertAliasesIds {
        self.0.ids = ids.into();

        InsertAliasesIds(self.0)
    }

    /// Assigns aliases to elements found using the search query.
    /// Equivalent to `ids(QueryIds::Search(search)/*...*/)`.
    pub fn search(mut self) -> Search<InsertAliasesQuery> {
        self.0.ids = QueryIds::Search(SearchQuery::new());
        Search(self.0)
    }
}

impl InsertAliasesIds {
    /// Returns the built `InsertAliasesQuery` object.
    pub fn query(self) -> InsertAliasesQuery {
        self.0
    }
}
