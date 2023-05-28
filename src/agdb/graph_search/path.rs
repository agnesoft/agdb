use crate::graph::GraphIndex;

#[derive(Clone)]
pub(crate) struct Path {
    pub(crate) elements: Vec<GraphIndex>,
    pub(crate) cost: u64,
}
