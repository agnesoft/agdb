use super::search_control::SearchControl;
use crate::graph::GraphIndex;

pub trait SearchHandler {
    fn process(&self, index: &GraphIndex, distance: &u64) -> SearchControl;
}
