use crate::query::select_node_count::SelectNodeCountQuery;

/// Select node count builder.
pub struct SelectNodeCount {}

impl SelectNodeCount {
    /// Returns the `SelectNodeCountQuery` object.
    pub fn query(self) -> SelectNodeCountQuery {
        SelectNodeCountQuery {}
    }
}
