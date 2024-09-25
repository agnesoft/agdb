use crate::query_builder::search::Search;
use crate::QueryIds;
use crate::RemoveValuesQuery;
use crate::SearchQuery;

/// Remove values builder that lets you select the ids from
/// which to remove the values.
pub struct RemoveValues(pub RemoveValuesQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct RemoveValuesIds(pub RemoveValuesQuery);

impl RemoveValues {
    /// Id, list of ids or search of the database elements to delete
    /// the values from. All of the ids must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> RemoveValuesIds {
        self.0 .0.ids = ids.into();

        RemoveValuesIds(self.0)
    }

    /// Remove the values from the elements found using the search query.
    /// Equivalent to `ids(QueryIds::Search(search)/*...*/)`.
    pub fn search(mut self) -> Search<RemoveValuesQuery> {
        self.0 .0.ids = QueryIds::Search(SearchQuery::new());
        Search(self.0)
    }
}

impl RemoveValuesIds {
    /// Returns the built `RemoveValuesQuery` object.
    pub fn query(self) -> RemoveValuesQuery {
        self.0
    }
}
