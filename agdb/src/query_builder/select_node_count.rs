use crate::query::select_node_count::SelectNodeCountQuery;

/// Select node count builder.
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
pub struct SelectNodeCount {}

impl SelectNodeCount {
    /// Returns the `SelectNodeCountQuery` object.
    pub fn query(self) -> SelectNodeCountQuery {
        SelectNodeCountQuery {}
    }
}
