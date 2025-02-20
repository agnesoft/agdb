use crate::InsertAliasesQuery;
use crate::QueryIds;

/// Insert aliases builder to select `ids`
/// of the aliases.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct InsertAliases(pub InsertAliasesQuery);

/// Final builder that lets you create
/// an actual query object.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct InsertAliasesIds(pub InsertAliasesQuery);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl InsertAliases {
    /// Ids of the db elements to be aliased. Only nodes can be aliased
    /// (positive ids) and the ids must exist in the database. NOTE: Search
    /// query in place of ids is not allowed and will be ignored if used.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> InsertAliasesIds {
        self.0.ids = ids.into();

        InsertAliasesIds(self.0)
    }
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl InsertAliasesIds {
    /// Returns the built `InsertAliasesQuery` object.
    pub fn query(self) -> InsertAliasesQuery {
        self.0
    }
}
