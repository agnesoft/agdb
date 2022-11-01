use super::search_control::SearchControl;
use crate::graph::graph_index::GraphIndex;

pub trait SearchHandler {
    fn process(&self, index: &GraphIndex, distance: &u64) -> SearchControl;
}
