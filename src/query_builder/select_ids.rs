use crate::query::select_query::SelectQuery;

/// Final builder that lets you create
/// an actual query object.
pub struct SelectIds(pub SelectQuery);

impl SelectIds {
    /// Returns the built `SelectQuery` object.
    pub fn query(self) -> SelectQuery {
        self.0
    }
}
