use agdb_graph::GraphIndex;

pub(crate) struct SearchIndex {
    pub(crate) index: GraphIndex,
    pub(crate) distance: u64,
}
