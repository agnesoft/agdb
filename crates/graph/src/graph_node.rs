use super::graph_data::GraphData;
use super::graph_edge_iterator::GraphEdgeIterator;
use super::graph_impl::GraphImpl;

pub struct GraphNode<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) index: i64,
}

impl<'a, Data> GraphNode<'a, Data>
where
    Data: GraphData,
{
    pub fn index(&self) -> i64 {
        self.index
    }

    pub fn edge_from_iter(&self) -> GraphEdgeIterator<Data> {
        GraphEdgeIterator {
            graph: self.graph,
            index: self.graph.first_edge_from(self.index).unwrap_or(0),
        }
    }
}
