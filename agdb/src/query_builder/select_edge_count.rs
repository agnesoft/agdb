use crate::QueryIds;
use crate::SelectEdgeCountQuery;

/// Select edge count builder.
pub struct SelectEdgeCount(pub SelectEdgeCountQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct SelectEdgeCountIds(pub SelectEdgeCountQuery);

impl SelectEdgeCount {
    /// An id or list of ids or search query to select edge count of.
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectEdgeCountIds {
        self.0.ids = ids.into();

        SelectEdgeCountIds(self.0)
    }
}

impl SelectEdgeCountIds {
    /// Returns the built `SelectEdgeCountQuery` object.
    pub fn query(self) -> SelectEdgeCountQuery {
        self.0
    }
}
