use crate::search_control::SearchControl;
use agdb_graph::GraphIndex;

pub trait SearchHandler {
    fn process(&self, index: &GraphIndex, distance: &u64) -> SearchControl;
}
