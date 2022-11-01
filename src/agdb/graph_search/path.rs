use crate::graph::graph_index::GraphIndex;

#[derive(Clone)]
pub(crate) struct Path {
    pub(crate) elements: Vec<GraphIndex>,
    pub(crate) cost: u64,
}
