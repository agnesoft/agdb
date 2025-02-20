use crate::query_builder::search::Search;
use crate::QueryIds;
use crate::SelectKeysQuery;

/// Select keys builder.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct SelectKeys(pub SelectKeysQuery);

/// Final builder that lets you create
/// an actual query object.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct SelectKeysIds(pub SelectKeysQuery);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SelectKeys {
    /// An id or list of ids or search query to select keys of.
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectKeysIds {
        self.0 .0 = ids.into();

        SelectKeysIds(self.0)
    }

    /// Select keys of elements returned from the search query.
    /// Equivalent to `ids(QueryBuilder::search()/* ... */)`.
    pub fn search(self) -> Search<SelectKeysQuery> {
        Search(SelectKeysQuery(QueryIds::Search(crate::SearchQuery::new())))
    }
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SelectKeysIds {
    /// Returns the built `SelectKeysQuery` object.
    pub fn query(self) -> SelectKeysQuery {
        self.0
    }
}
