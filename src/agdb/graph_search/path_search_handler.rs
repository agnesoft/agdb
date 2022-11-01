use crate::graph::graph_index::GraphIndex;

pub trait PathSearchHandler {
    fn process(&self, index: &GraphIndex, distance: &u64) -> u64;
}
