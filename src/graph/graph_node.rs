use super::Graph;

#[allow(dead_code)]
pub(crate) struct GraphNode<'a> {
    pub(crate) graph: &'a Graph,
    pub(crate) index: i64,
}

#[allow(dead_code)]
impl<'a> GraphNode<'a> {
    pub(crate) fn index(&self) -> i64 {
        self.index
    }
}
