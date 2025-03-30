use crate::QueryIds;
use crate::SelectValuesQuery;
use crate::query_builder::search::Search;

/// Select values builder.
pub struct SelectValues(pub SelectValuesQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct SelectValuesIds(pub SelectValuesQuery);

impl SelectValues {
    /// An id or list of ids or search query to select values of.
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectValuesIds {
        self.0.ids = ids.into();

        SelectValuesIds(self.0)
    }

    /// Select using the result of a search query as ids.
    /// Equivalent to `ids(QueryBuilder::search()/* ... */)`.
    pub fn search(mut self) -> Search<SelectValuesQuery> {
        self.0.ids = QueryIds::Search(crate::SearchQuery::new());
        Search(self.0)
    }
}

impl SelectValuesIds {
    /// Returns the built `SelectValuesQuery` object.
    pub fn query(self) -> SelectValuesQuery {
        self.0
    }
}
