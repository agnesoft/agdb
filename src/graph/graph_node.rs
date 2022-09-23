use super::graph_edge_iterator::GraphEdgeIterator;
use super::GraphImpl;

#[allow(dead_code)]
pub(crate) struct GraphNode<'a> {
    pub(super) graph: &'a GraphImpl,
    pub(super) index: i64,
}

#[allow(dead_code)]
impl<'a> GraphNode<'a> {
    pub(crate) fn index(&self) -> i64 {
        self.index
    }

    pub(crate) fn edge_from_iter(&self) -> GraphEdgeIterator {
        GraphEdgeIterator {
            graph: self.graph,
            index: self.graph.first_edge_from(self.index),
        }
    }
}

impl<'a> PartialEq for GraphNode<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.graph as *const _ == other.graph as *const _ && self.index == other.index
    }
}

impl<'a> std::fmt::Debug for GraphNode<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphNode")
            .field("graph", &(self.graph as *const _))
            .field("index", &self.index)
            .finish()
    }
}
