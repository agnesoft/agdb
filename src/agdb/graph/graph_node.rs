use super::graph_data::GraphData;
use super::graph_edge_iterator::GraphEdgeIterator;
use super::graph_edge_reverse_iterator::GraphEdgeReverseIterator;
use super::graph_impl::GraphImpl;
use super::graph_index::GraphIndex;

pub struct GraphNode<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) index: GraphIndex,
}

#[allow(dead_code)]
impl<'a, Data> GraphNode<'a, Data>
where
    Data: GraphData,
{
    pub fn index(&self) -> GraphIndex {
        self.index.clone()
    }

    pub fn edge_iter_from(&self) -> GraphEdgeIterator<Data> {
        GraphEdgeIterator {
            graph: self.graph,
            index: self.graph.first_edge_from(&self.index).unwrap_or_default(),
        }
    }

    pub fn edge_iter_to(&self) -> GraphEdgeReverseIterator<Data> {
        GraphEdgeReverseIterator {
            graph: self.graph,
            index: self.graph.first_edge_to(&self.index).unwrap_or_default(),
        }
    }
}
