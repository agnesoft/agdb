use crate::query::remove_aliases_query::RemoveAliasesQuery;

/// Final builder that lets you create
/// an actual query object.
pub struct RemoveAliases(pub RemoveAliasesQuery);

impl RemoveAliases {
    /// Returns the built `RemoveAliasesQuery` object.
    pub fn query(self) -> RemoveAliasesQuery {
        self.0
    }
}
