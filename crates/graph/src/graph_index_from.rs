use crate::graph_index::GraphIndex;

impl From<i64> for GraphIndex {
    fn from(index: i64) -> Self {
        Self { index }
    }
}
