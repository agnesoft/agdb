use crate::RemoveAliasesQuery;

/// Final builder that lets you create
/// an actual query object.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct RemoveAliases(pub RemoveAliasesQuery);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl RemoveAliases {
    /// Returns the built `RemoveAliasesQuery` object.
    pub fn query(self) -> RemoveAliasesQuery {
        self.0
    }
}
