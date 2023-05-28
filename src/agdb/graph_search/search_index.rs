use crate::graph::GraphIndex;

#[derive(Clone, Copy)]
pub(crate) struct SearchIndex {
    pub(crate) index: GraphIndex,
    pub(crate) distance: u64,
}
