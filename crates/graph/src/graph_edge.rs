use super::graph_data::GraphData;
use super::graph_impl::GraphImpl;

pub struct GraphEdge<'a, Data>
where
    Data: GraphData,
{
    #[allow(dead_code)]
    pub(crate) graph: &'a GraphImpl<Data>,
    pub(crate) index: i64,
}

impl<'a, Data> GraphEdge<'a, Data>
where
    Data: GraphData,
{
    pub fn index(&self) -> i64 {
        self.index
    }
}
