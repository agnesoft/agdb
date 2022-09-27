use super::graph_data::GraphData;
use super::graph_impl::GraphImpl;
use super::graph_node::GraphNode;

pub(crate) struct GraphNodeIterator<'a, Data: GraphData> {
    pub(super) graph: &'a GraphImpl<Data>,
    pub(super) index: i64,
}

impl<'a, Data: GraphData> Iterator for GraphNodeIterator<'a, Data> {
    type Item = GraphNode<'a, Data>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.graph.next_node(self.index).unwrap_or(None) {
            self.index = next;
            return Some(GraphNode {
                graph: self.graph,
                index: self.index,
            });
        }

        None
    }
}
