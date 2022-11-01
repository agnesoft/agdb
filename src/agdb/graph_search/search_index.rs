use crate::graph::graph_index::GraphIndex;

pub(crate) struct SearchIndex {
    pub(crate) index: GraphIndex,
    pub(crate) distance: u64,
}
