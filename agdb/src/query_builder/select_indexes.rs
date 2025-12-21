use crate::SelectIndexesQuery;

/// Select indexes builder.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct SelectIndexes {}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SelectIndexes {
    /// Returns the built `SelectIndexesQuery`.
    pub fn query(&self) -> SelectIndexesQuery {
        SelectIndexesQuery {}
    }
}
