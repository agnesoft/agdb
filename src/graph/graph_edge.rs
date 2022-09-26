use super::Graph;

#[allow(dead_code)]
pub(crate) struct GraphEdge<'a> {
    pub(super) graph: &'a Graph,
    pub(super) index: i64,
}

#[allow(dead_code)]
impl<'a> GraphEdge<'a> {
    pub(crate) fn index(&self) -> i64 {
        self.index
    }
}
