use super::graph_data::GraphData;
use super::graph_edge::GraphEdge;
use super::graph_impl::GraphImpl;
use super::graph_index::GraphIndex;

pub struct GraphEdgeReverseIterator<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) index: GraphIndex,
}

impl<'a, Data> Iterator for GraphEdgeReverseIterator<'a, Data>
where
    Data: GraphData,
{
    type Item = GraphEdge<'a, Data>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.index.is_valid() {
            return None;
        }

        let current_index = self.index.clone();

        self.index = self.graph.next_edge_to(&self.index).unwrap_or_default();

        Some(GraphEdge {
            graph: self.graph,
            index: current_index,
        })
    }
}
