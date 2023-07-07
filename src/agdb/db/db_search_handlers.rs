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
    fn process(&mut self, index: GraphIndex, distance: u64) -> Result<SearchControl, DbError> {
        self.db
            .evaluate_conditions(index, distance, self.conditions)
    }
}

impl<'a> SearchHandler for LimitHandler<'a> {
    fn process(&mut self, index: GraphIndex, distance: u64) -> Result<SearchControl, DbError> {
        let control = self
            .db
            .evaluate_conditions(index, distance, self.conditions)?;
        let add = control.is_true();

        if add {
            self.counter += 1;
        }

        if self.counter == self.limit {
            Ok(SearchControl::Finish(add))
        } else {
            Ok(control)
        }
    }
}

impl<'a> SearchHandler for OffsetHandler<'a> {
    fn process(&mut self, index: GraphIndex, distance: u64) -> Result<SearchControl, DbError> {
        let mut control = self
            .db
            .evaluate_conditions(index, distance, self.conditions)?;

        if control.is_true() {
            self.counter += 1;
            control.set_value(self.offset < self.counter);
        }

        Ok(control)
    }
}

impl<'a> SearchHandler for LimitOffsetHandler<'a> {
    fn process(&mut self, index: GraphIndex, distance: u64) -> Result<SearchControl, DbError> {
        let mut control = self
            .db
            .evaluate_conditions(index, distance, self.conditions)?;

        if control.is_true() {
            self.counter += 1;
            control.set_value(self.offset < self.counter);
        }

        if self.counter == self.limit {
            Ok(SearchControl::Finish(control.is_true()))
        } else {
            Ok(control)
        }
    }
}

impl<'a> PathSearchHandler for PathHandler<'a> {
    fn process(&self, index: GraphIndex, distance: u64) -> Result<(u64, bool), DbError> {
        match self
            .db
            .evaluate_conditions(index, distance, self.conditions)?
        {
            SearchControl::Continue(add) => Ok((1 + (!add as u64), add)),
            SearchControl::Finish(add) | SearchControl::Stop(add) => Ok((0, add)),
        }
    }
}
