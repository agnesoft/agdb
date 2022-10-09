use super::graph_data::GraphData;
use super::graph_impl::GraphImpl;

#[allow(dead_code)]
pub(crate) struct GraphEdge<'a, Data: GraphData> {
    pub(super) graph: &'a GraphImpl<Data>,
    pub(super) index: i64,
}

#[allow(dead_code)]
impl<'a, Data: GraphData> GraphEdge<'a, Data> {
    pub(crate) fn index(&self) -> i64 {
        self.index
    }
}
