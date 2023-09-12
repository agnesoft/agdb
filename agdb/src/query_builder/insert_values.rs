use crate::InsertValuesQuery;
use crate::QueryIds;

/// Insert values builder to set ids to which the values
/// should be inserted.
pub struct InsertValues(pub InsertValuesQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct InsertValuesIds(pub InsertValuesQuery);

impl InsertValues {
    /// An id or list of ids or search query from to which to insert the values.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> InsertValuesIds {
        self.0.ids = ids.into();

        InsertValuesIds(self.0)
    }
}

impl InsertValuesIds {
    /// Returns the built `InsertValuesQuery` object.
    pub fn query(self) -> InsertValuesQuery {
        self.0
    }
}
