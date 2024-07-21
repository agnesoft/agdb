use crate::SelectValuesQuery;

/// Final builder that lets you create
/// an actual query object.
pub struct SelectIds(pub SelectValuesQuery);

impl SelectIds {
    /// Returns the built `SelectQuery` object.
    pub fn query(self) -> SelectValuesQuery {
        self.0
    }
}
