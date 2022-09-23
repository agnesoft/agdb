use super::graph_node::GraphNode;
use super::GraphImpl;

pub(crate) struct GraphNodeIterator<'a> {
    pub(super) graph: &'a GraphImpl,
    pub(super) index: i64,
}

impl<'a> Iterator for GraphNodeIterator<'a> {
    type Item = GraphNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.graph.next_node(self.index) {
            self.index = next;
            return Some(GraphNode {
                graph: self.graph,
                index: self.index,
            });
        }

        None
    }
}
