use super::graph_data::GraphData;
use super::graph_edge_iterator::GraphEdgeIterator;
use super::graph_impl::GraphImpl;

#[allow(dead_code)]
pub(crate) struct GraphNode<'a, Data: GraphData> {
    pub(super) graph: &'a GraphImpl<Data>,
    pub(super) index: i64,
}

#[allow(dead_code)]
impl<'a, Data: GraphData> GraphNode<'a, Data> {
    pub(crate) fn index(&self) -> i64 {
        self.index
    }

    pub(crate) fn edge_from_iter(&self) -> GraphEdgeIterator<Data> {
        GraphEdgeIterator {
            graph: self.graph,
            index: self.graph.first_edge_from(self.index).unwrap_or(0),
        }
    }
}
