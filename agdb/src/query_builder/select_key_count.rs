use crate::query_builder::search::Search;
use crate::QueryIds;
use crate::SelectKeyCountQuery;

/// Select key count builder.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct SelectKeyCount(pub SelectKeyCountQuery);

/// Final builder that lets you create
/// an actual query object.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct SelectKeyCountIds(pub SelectKeyCountQuery);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SelectKeyCount {
    /// An id or list of ids or search query to select key count of.
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectKeyCountIds {
        self.0 .0 = ids.into();

        SelectKeyCountIds(self.0)
    }

    /// Select key count for elements returned from the search query.
    /// Equivalent to `ids(QueryBuilder::search()/* ... */)`.
    pub fn search(self) -> Search<SelectKeyCountQuery> {
        Search(SelectKeyCountQuery(QueryIds::Search(
            crate::SearchQuery::new(),
        )))
    }
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SelectKeyCountIds {
    /// Returns the built `SelectKeyCountQuery` object.
    pub fn query(self) -> SelectKeyCountQuery {
        self.0
    }
}
