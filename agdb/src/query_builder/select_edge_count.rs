use crate::QueryIds;
use crate::SelectEdgeCountQuery;
use crate::query_builder::search::Search;

/// Select edge count builder.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct SelectEdgeCount(pub SelectEdgeCountQuery);

/// Final builder that lets you create
/// an actual query object.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct SelectEdgeCountIds(pub SelectEdgeCountQuery);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SelectEdgeCount {
    /// An id or list of ids or search query to select edge count of.
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectEdgeCountIds {
        self.0.ids = ids.into();

        SelectEdgeCountIds(self.0)
    }

    /// Select keys of elements returned from the search query.
    /// Equivalent to `ids(QueryBuilder::search()/* ... */)`.
    pub fn search(mut self) -> Search<SelectEdgeCountQuery> {
        self.0.ids = QueryIds::Search(crate::SearchQuery::new());
        Search(self.0)
    }
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SelectEdgeCountIds {
    /// Returns the built `SelectEdgeCountQuery` object.
    pub fn query(self) -> SelectEdgeCountQuery {
        self.0
    }
}
