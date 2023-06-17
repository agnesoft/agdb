use super::db_error::DbError;
use crate::graph::GraphIndex;
use crate::graph_search::PathSearchHandler;
use crate::graph_search::SearchControl;
use crate::graph_search::SearchHandler;
use crate::query::query_condition::QueryCondition;
use crate::Db;

pub(crate) struct DefaultHandler<'a> {
    db: &'a Db,
    conditions: &'a Vec<QueryCondition>,
}

pub(crate) struct LimitHandler<'a> {
    limit: u64,
    counter: u64,
    db: &'a Db,
    conditions: &'a Vec<QueryCondition>,
}

pub(crate) struct OffsetHandler<'a> {
    offset: u64,
    counter: u64,
    db: &'a Db,
    conditions: &'a Vec<QueryCondition>,
}

pub(crate) struct LimitOffsetHandler<'a> {
    limit: u64,
    offset: u64,
    counter: u64,
    db: &'a Db,
    conditions: &'a Vec<QueryCondition>,
}

pub(crate) struct PathHandler<'a> {
    db: &'a Db,
    conditions: &'a Vec<QueryCondition>,
}

impl<'a> DefaultHandler<'a> {
    pub(crate) fn new(db: &'a Db, conditions: &'a Vec<QueryCondition>) -> Self {
        Self { db, conditions }
    }
}

impl<'a> LimitHandler<'a> {
    pub fn new(limit: u64, db: &'a Db, conditions: &'a Vec<QueryCondition>) -> Self {
        Self {
            limit,
            counter: 0,
            db,
            conditions,
        }
    }
}

impl<'a> OffsetHandler<'a> {
    pub fn new(offset: u64, db: &'a Db, conditions: &'a Vec<QueryCondition>) -> Self {
        Self {
            offset,
            counter: 0,
            db,
            conditions,
        }
    }
}

impl<'a> LimitOffsetHandler<'a> {
    pub fn new(limit: u64, offset: u64, db: &'a Db, conditions: &'a Vec<QueryCondition>) -> Self {
        Self {
            limit: limit + offset,
            offset,
            counter: 0,
            db,
            conditions,
        }
    }
}

impl<'a> PathHandler<'a> {
    pub fn new(db: &'a Db, conditions: &'a Vec<QueryCondition>) -> Self {
        Self { db, conditions }
    }
}

impl<'a> SearchHandler for DefaultHandler<'a> {
    fn process(&mut self, index: GraphIndex, distance: u64) -> SearchControl {
        self.db
            .evaluate_conditions(index, distance, self.conditions)
    }
}

impl<'a> SearchHandler for LimitHandler<'a> {
    fn process(&mut self, index: GraphIndex, distance: u64) -> SearchControl {
        match self
            .db
            .evaluate_conditions(index, distance, self.conditions)
        {
            SearchControl::Continue(add) => {
                if add {
                    self.counter += 1
                }

                if self.counter == self.limit {
                    SearchControl::Finish(add)
                } else {
                    SearchControl::Continue(add)
                }
            }
            SearchControl::Stop(add) => {
                if add {
                    self.counter += 1;
                }

                if self.counter == self.limit {
                    SearchControl::Finish(add)
                } else {
                    SearchControl::Stop(add)
                }
            }
            SearchControl::Finish(_) => SearchControl::Finish(false),
        }
    }
}

impl<'a> SearchHandler for OffsetHandler<'a> {
    fn process(&mut self, index: GraphIndex, distance: u64) -> SearchControl {
        match self
            .db
            .evaluate_conditions(index, distance, self.conditions)
        {
            SearchControl::Continue(add) => {
                if add {
                    self.counter += 1
                }
                SearchControl::Continue(add && self.offset < self.counter)
            }
            SearchControl::Stop(add) => {
                if add {
                    self.counter += 1;
                }
                SearchControl::Stop(add && self.offset < self.counter)
            }
            SearchControl::Finish(_) => SearchControl::Finish(false),
        }
    }
}

impl<'a> SearchHandler for LimitOffsetHandler<'a> {
    fn process(&mut self, index: GraphIndex, distance: u64) -> SearchControl {
        match self
            .db
            .evaluate_conditions(index, distance, self.conditions)
        {
            SearchControl::Continue(add) => {
                if add {
                    self.counter += 1
                }

                if self.counter == self.limit {
                    SearchControl::Finish(add && self.offset < self.counter)
                } else {
                    SearchControl::Continue(add && self.offset < self.counter)
                }
            }
            SearchControl::Stop(add) => {
                if add {
                    self.counter += 1;
                }

                if self.counter == self.limit {
                    SearchControl::Finish(add && self.offset < self.counter)
                } else {
                    SearchControl::Stop(add && self.offset < self.counter)
                }
            }
            SearchControl::Finish(_) => SearchControl::Finish(false),
        }
    }
}

impl<'a> PathSearchHandler for PathHandler<'a> {
    fn process(&self, index: GraphIndex, distance: u64) -> u64 {
        self.db
            .evaluate_path_conditions(index, distance, self.conditions)
    }
}
