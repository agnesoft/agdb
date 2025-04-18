use crate::QueryIds;
use crate::SelectKeyCountQuery;
use crate::query_builder::search::Search;

/// Select key count builder.
pub struct SelectKeyCount(pub SelectKeyCountQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct SelectKeyCountIds(pub SelectKeyCountQuery);

impl SelectKeyCount {
    /// An id or list of ids or search query to select key count of.
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectKeyCountIds {
        self.0.0 = ids.into();

        SelectKeyCountIds(self.0)
    }

    /// Select key count for elements returned from the search query.
    /// Equivalent to `ids(QueryBuilder::search()/* ... */)`.
    pub fn search(self) -> Search<SelectKeyCountQuery> {
        Search(SelectKeyCountQuery(QueryIds::Search(
            crate::SearchQuery::new(),
        )))
    }
}

impl SelectKeyCountIds {
    /// Returns the built `SelectKeyCountQuery` object.
    pub fn query(self) -> SelectKeyCountQuery {
        self.0
    }
}
