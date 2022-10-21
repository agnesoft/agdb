use agdb_graph::GraphIndex;

#[derive(Default, Clone)]
pub(crate) struct Path {
    pub(crate) elements: Vec<GraphIndex>,
    pub(crate) cost: u64,
}
