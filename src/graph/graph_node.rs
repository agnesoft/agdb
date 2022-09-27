use super::graph_edge_iterator::GraphEdgeIterator;
use super::Graph;

#[allow(dead_code)]
pub(crate) struct GraphNode<'a> {
    pub(super) graph: &'a Graph,
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
