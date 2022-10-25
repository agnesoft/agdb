use crate::graph_data::GraphData;
use crate::graph_impl::GraphImpl;
use crate::graph_index::GraphIndex;

pub struct GraphEdge<'a, Data>
where
    Data: GraphData,
{
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) index: GraphIndex,
}

impl<'a, Data> GraphEdge<'a, Data>
where
    Data: GraphData,
{
    pub fn index(&self) -> GraphIndex {
        self.index.clone()
    }

    pub fn index_from(&self) -> GraphIndex {
        self.graph.edge_from(&self.index)
    }

    pub fn index_to(&self) -> GraphIndex {
        self.graph.edge_to(&self.index)
    }
}
