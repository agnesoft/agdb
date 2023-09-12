use crate::QueryIds;
use crate::SelectValuesQuery;

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
}

impl SelectValuesIds {
    /// Returns the built `SelectValuesQuery` object.
    pub fn query(self) -> SelectValuesQuery {
        self.0
    }
}
