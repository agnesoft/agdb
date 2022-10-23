use agdb_graph::GraphIndex;

#[derive(Clone)]
pub(crate) struct Path {
    pub(crate) elements: Vec<GraphIndex>,
    pub(crate) cost: u64,
}
