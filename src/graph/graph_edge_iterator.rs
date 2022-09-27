use super::graph_data::GraphData;
use super::graph_edge::GraphEdge;
use super::graph_impl::GraphImpl;

pub(crate) struct GraphEdgeIterator<'a, Data: GraphData> {
    pub(super) graph: &'a GraphImpl<Data>,
    pub(super) index: i64,
}

impl<'a, Data: GraphData> Iterator for GraphEdgeIterator<'a, Data> {
    type Item = GraphEdge<'a, Data>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            return None;
        }

        let index = self.index;
        self.index = self.graph.next_edge_from(self.index).unwrap_or(0);

        Some(GraphEdge {
            graph: self.graph,
            index,
        })
    }
}
