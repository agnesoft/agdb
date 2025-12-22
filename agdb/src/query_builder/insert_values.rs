use crate::InsertValuesQuery;
use crate::QueryIds;
use crate::SearchQuery;
use crate::query_builder::search::Search;

/// Insert values builder to set ids to which the values
/// should be inserted.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct InsertValues(pub InsertValuesQuery);

/// Final builder that lets you create
/// an actual query object.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct InsertValuesIds(pub InsertValuesQuery);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl InsertValues {
    /// An id or list of ids or search query from to which to insert the values.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> InsertValuesIds {
        self.0.ids = ids.into();

        InsertValuesIds(self.0)
    }

    /// Inserts values into elements found using the search query.
    /// Equivalent to `ids(QueryIds::Search(search)/*...*/)`.
    pub fn search(mut self) -> Search<InsertValuesQuery> {
        self.0.ids = QueryIds::Search(SearchQuery::new());
        Search(self.0)
    }
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl InsertValuesIds {
    /// Returns the built `InsertValuesQuery` object.
    pub fn query(self) -> InsertValuesQuery {
        self.0
    }
}
