use crate::SelectIndexesQuery;

/// Select indexes builder.
pub struct SelectIndexes {}

impl SelectIndexes {
    /// Returns the built `SelectIndexesQuery`.
    pub fn query(&self) -> SelectIndexesQuery {
        SelectIndexesQuery {}
    }
}
