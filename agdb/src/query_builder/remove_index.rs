use crate::DbValue;
use crate::RemoveIndexQuery;

/// Final step in the remove index query builder.
pub struct RemoveIndex(pub DbValue);

impl RemoveIndex {
    /// Returns the built `RemoveIndexQuery`.
    pub fn query(self) -> RemoveIndexQuery {
        RemoveIndexQuery(self.0)
    }
}
