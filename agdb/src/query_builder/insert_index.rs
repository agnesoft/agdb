use crate::DbValue;
use crate::InsertIndexQuery;

/// Final step in the insert index query builder.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct InsertIndex(pub DbValue);

impl InsertIndex {
    /// Returns the built `InsertIndexQuery`.
    pub fn query(self) -> InsertIndexQuery {
        InsertIndexQuery(self.0)
    }
}
