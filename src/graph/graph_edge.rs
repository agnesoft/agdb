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

impl<'a> PartialEq for GraphEdge<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.graph as *const _ == other.graph as *const _ && self.index == other.index
    }
}

impl<'a> std::fmt::Debug for GraphEdge<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphEdge")
            .field("graph", &(self.graph as *const _))
            .field("index", &self.index)
            .finish()
    }
}
