use crate::DbValue;
use crate::InsertIndexQuery;

/// Final step in the insert index query builder.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct InsertIndex(pub DbValue);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl InsertIndex {
    /// Returns the built `InsertIndexQuery`.
    pub fn query(self) -> InsertIndexQuery {
        InsertIndexQuery(self.0)
    }
}
