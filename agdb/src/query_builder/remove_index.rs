use crate::DbValue;
use crate::RemoveIndexQuery;

/// Final step in the remove index query builder.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[cfg_attr(feature = "api", type_def(inherent))]
pub struct RemoveIndex(pub DbValue);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl RemoveIndex {
    /// Returns the built `RemoveIndexQuery`.
    pub fn query(self) -> RemoveIndexQuery {
        RemoveIndexQuery(self.0)
    }
}
