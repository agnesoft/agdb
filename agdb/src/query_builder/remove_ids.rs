use crate::query::remove_query::RemoveQuery;

/// Final builder that lets you create
/// an actual query object.
pub struct RemoveIds(pub RemoveQuery);

impl RemoveIds {
    /// Returns the built `RemoveQuery` object.
    pub fn query(self) -> RemoveQuery {
        self.0
    }
}
