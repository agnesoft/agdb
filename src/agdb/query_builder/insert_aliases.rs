use crate::query::insert_aliases_query::InsertAliasesQuery;
use crate::query::query_ids::QueryIds;

/// Insert aliases builder to select `ids`
/// of the aliases.
pub struct InsertAliases(pub InsertAliasesQuery);

/// Final builder that lets you create
/// an actual query object.
pub struct InsertAliasesIds(pub InsertAliasesQuery);

impl InsertAliases {
    /// Ids of the db elements to be aliased. Only nodes can be aliased
    /// (positive ids) and the ids must exist in the database. NOTE: Search
    /// query in place of ids is not allowed and will be ignored if used.
    ///
    /// Options:
    ///
    /// ```
    /// use agdb::QueryBuilder;
    ///
    /// QueryBuilder::insert().aliases("a").ids(1).query();
    /// ```
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> InsertAliasesIds {
        self.0.ids = ids.into();

        InsertAliasesIds(self.0)
    }
}

impl InsertAliasesIds {
    /// Returns the built `InsertAliasesQuery` object.
    pub fn query(self) -> InsertAliasesQuery {
        self.0
    }
}
