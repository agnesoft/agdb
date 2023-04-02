use super::{graph_data::GraphData, graph_impl::GraphImpl, graph_index::GraphIndex};

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
        self.index
    }

    pub fn index_from(&self) -> GraphIndex {
        self.graph.edge_from(&self.index)
    }

    pub fn index_to(&self) -> GraphIndex {
        self.graph.edge_to(&self.index)
    }
}
