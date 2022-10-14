use agdb_graph::GraphIndex;

pub(crate) struct GraphSearchIndex {
    pub(crate) index: GraphIndex,
    pub(crate) distance: u64,
}
