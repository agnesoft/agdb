use crate::graph_data::GraphData;
use crate::graph_impl::GraphImpl;
use crate::graph_index::GraphIndex;
use crate::graph_node::GraphNode;

pub struct GraphNodeIterator<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) index: GraphIndex,
}

impl<'a, Data> Iterator for GraphNodeIterator<'a, Data>
where
    Data: GraphData,
{
    type Item = GraphNode<'a, Data>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.graph.next_node(&self.index).unwrap_or(None) {
            self.index = next;
            return Some(GraphNode {
                graph: self.graph,
                index: self.index.clone(),
            });
        }

        None
    }
}
