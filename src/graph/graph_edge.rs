use super::GraphImpl;

#[allow(dead_code)]
pub(crate) struct GraphEdge<'a> {
    pub(super) graph: &'a GraphImpl,
    pub(super) index: i64,
}

#[allow(dead_code)]
impl<'a> GraphEdge<'a> {
    pub(crate) fn index(&self) -> i64 {
        self.index
    }
}
