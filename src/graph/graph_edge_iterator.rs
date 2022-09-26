use super::graph_edge::GraphEdge;
use super::Graph;

pub(crate) struct GraphEdgeIterator<'a> {
    pub(super) graph: &'a Graph,
    pub(super) index: i64,
}

impl<'a> Iterator for GraphEdgeIterator<'a> {
    type Item = GraphEdge<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == 0 {
            return None;
        }

        let index = self.index;
        self.index = self.graph.next_edge_from(self.index);

        Some(GraphEdge {
            graph: self.graph,
            index,
        })
    }
}
