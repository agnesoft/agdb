use crate::graph::GraphIndex;
use crate::graph_search::PathSearchHandler;
use crate::graph_search::SearchControl;
use crate::graph_search::SearchHandler;

pub(crate) struct DefaultHandler {}

pub(crate) struct LimitHandler {
    limit: u64,
    counter: u64,
}

pub(crate) struct PathHandler {}

impl SearchHandler for DefaultHandler {
    fn process(&mut self, _index: GraphIndex, _distance: u64) -> SearchControl {
        SearchControl::Continue(true)
    }
}

impl LimitHandler {
    pub fn new(limit: u64) -> Self {
        Self { limit, counter: 0 }
    }
}

impl SearchHandler for LimitHandler {
    fn process(&mut self, _index: GraphIndex, _distance: u64) -> SearchControl {
        self.counter += 1;

        if self.counter == self.limit {
            SearchControl::Finish(true)
        } else {
            SearchControl::Continue(true)
        }
    }
}

impl PathSearchHandler for PathHandler {
    fn process(&self, _index: GraphIndex, _distance: u64) -> u64 {
        1
    }
}
