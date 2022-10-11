use crate::graph_data::GraphData;
use crate::graph_edge::GraphEdge;
use crate::graph_impl::GraphImpl;
use crate::graph_index::GraphIndex;

pub struct GraphEdgeIterator<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) index: GraphIndex,
}

impl<'a, Data> Iterator for GraphEdgeIterator<'a, Data>
where
    Data: GraphData,
{
    type Item = GraphEdge<'a, Data>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.index.is_valid() {
            return None;
        }

        let current_index = self.index.clone();
        self.index = self
            .graph
            .next_edge_from(&self.index)
            .unwrap_or(GraphIndex::default());

        Some(GraphEdge {
            graph: self.graph,
            index: current_index,
        })
    }
}
