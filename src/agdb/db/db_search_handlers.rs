use super::db_error::DbError;
use crate::graph::GraphIndex;
use crate::graph_search::PathSearchHandler;
use crate::graph_search::SearchControl;
use crate::graph_search::SearchHandler;

pub(crate) struct DefaultHandler {}

pub(crate) struct LimitHandler {
    limit: u64,
    counter: u64,
}

pub(crate) struct OffsetHandler {
    offset: u64,
    counter: u64,
}

pub(crate) struct LimitOffsetHandler {
    limit: u64,
    offset: u64,
    counter: u64,
}

pub(crate) struct PathHandler {}

impl SearchHandler for DefaultHandler {
    fn process(&mut self, _index: GraphIndex, _distance: u64) -> Result<SearchControl, DbError> {
        Ok(SearchControl::Continue(true))
    }
}

impl LimitHandler {
    pub fn new(limit: u64) -> Self {
        Self { limit, counter: 0 }
    }
}

impl OffsetHandler {
    pub fn new(offset: u64) -> Self {
        Self { offset, counter: 0 }
    }
}

impl LimitOffsetHandler {
    pub fn new(limit: u64, offset: u64) -> Self {
        Self {
            limit: limit + offset,
            offset,
            counter: 0,
        }
    }
}

impl SearchHandler for LimitHandler {
    fn process(&mut self, _index: GraphIndex, _distance: u64) -> Result<SearchControl, DbError> {
        self.counter += 1;

        if self.counter == self.limit {
            Ok(SearchControl::Finish(true))
        } else {
            Ok(SearchControl::Continue(true))
        }
    }
}

impl SearchHandler for OffsetHandler {
    fn process(&mut self, _index: GraphIndex, _distance: u64) -> Result<SearchControl, DbError> {
        self.counter += 1;
        Ok(SearchControl::Continue(self.offset < self.counter))
    }
}

impl SearchHandler for LimitOffsetHandler {
    fn process(&mut self, _index: GraphIndex, _distance: u64) -> Result<SearchControl, DbError> {
        self.counter += 1;

        if self.counter == self.limit {
            Ok(SearchControl::Finish(self.offset < self.counter))
        } else {
            Ok(SearchControl::Continue(self.offset < self.counter))
        }
    }
}

impl PathSearchHandler for PathHandler {
    fn process(&self, _index: GraphIndex, _distance: u64) -> Result<u64, DbError> {
        Ok(1)
    }
}
