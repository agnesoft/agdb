use crate::SelectValuesQuery;

/// Final builder that lets you create
/// an actual query object.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[cfg_attr(feature = "api", type_def(inherent))]
pub struct SelectIds(pub SelectValuesQuery);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SelectIds {
    /// Returns the built `SelectQuery` object.
    pub fn query(self) -> SelectValuesQuery {
        self.0
    }
}
