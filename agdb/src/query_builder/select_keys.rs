use crate::QueryIds;
use crate::SelectKeysQuery;

/// Select keys builder.
pub struct SelectKeys(pub SelectKeysQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct SelectKeysIds(pub SelectKeysQuery);

impl SelectKeys {
    /// An id or list of ids or search query to select keys of.
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectKeysIds {
        self.0 .0 = ids.into();

        SelectKeysIds(self.0)
    }
}

impl SelectKeysIds {
    /// Returns the built `SelectKeysQuery` object.
    pub fn query(self) -> SelectKeysQuery {
        self.0
    }
}
