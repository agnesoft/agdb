use crate::query::query_ids::QueryIds;
use crate::query::select_aliases_query::SelectAliasesQuery;
use crate::query::select_all_aliases_query::SelectAllAliases;

/// Select aliases builder.
pub struct SelectAliases(pub SelectAliasesQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct SelectAliasesIds(pub SelectAliasesQuery);

impl SelectAliases {
    /// An id or list of ids or search query to select aliases of.
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectAliasesIds {
        self.0 .0 = ids.into();

        SelectAliasesIds(self.0)
    }

    /// Returns the built `SelectAllAliases` object.
    pub fn query(self) -> SelectAllAliases {
        SelectAllAliases {}
    }
}

impl SelectAliasesIds {
    /// Returns the built `SelectAllAliases` object.
    pub fn query(self) -> SelectAliasesQuery {
        self.0
    }
}
