use crate::RemoveQuery;

/// Final builder that lets you create
/// an actual query object.
#[cfg_attr(feature = "api", derive(agdb::ApiDefImpl))]
pub struct RemoveIds(pub RemoveQuery);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl RemoveIds {
    /// Returns the built `RemoveQuery` object.
    pub fn query(self) -> RemoveQuery {
        self.0
    }
}
