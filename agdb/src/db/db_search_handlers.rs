use crate::db::DbImpl;
use crate::graph::GraphIndex;
use crate::graph_search::path_search::PathSearchHandler;
use crate::graph_search::SearchControl;
use crate::graph_search::SearchHandler;
use crate::query::query_condition::QueryCondition;
use crate::storage::StorageData;
use crate::DbError;

pub(crate) struct DefaultHandler<'a, Store: StorageData> {
    db: &'a DbImpl<Store>,
    conditions: &'a Vec<QueryCondition>,
}

pub(crate) struct LimitHandler<'a, Store: StorageData> {
    limit: u64,
    counter: u64,
    db: &'a DbImpl<Store>,
    conditions: &'a Vec<QueryCondition>,
}

pub(crate) struct OffsetHandler<'a, Store: StorageData> {
    offset: u64,
    counter: u64,
    db: &'a DbImpl<Store>,
    conditions: &'a Vec<QueryCondition>,
}

pub(crate) struct LimitOffsetHandler<'a, Store: StorageData> {
    limit: u64,
    offset: u64,
    counter: u64,
    db: &'a DbImpl<Store>,
    conditions: &'a Vec<QueryCondition>,
}

pub(crate) struct PathHandler<'a, Store: StorageData> {
    db: &'a DbImpl<Store>,
    conditions: &'a Vec<QueryCondition>,
}

impl<'a, Store: StorageData> DefaultHandler<'a, Store> {
    pub(crate) fn new(db: &'a DbImpl<Store>, conditions: &'a Vec<QueryCondition>) -> Self {
        Self { db, conditions }
    }
}

impl<'a, Store: StorageData> LimitHandler<'a, Store> {
    pub fn new(limit: u64, db: &'a DbImpl<Store>, conditions: &'a Vec<QueryCondition>) -> Self {
        Self {
            limit,
            counter: 0,
            db,
            conditions,
        }
    }
}

impl<'a, Store: StorageData> OffsetHandler<'a, Store> {
    pub fn new(offset: u64, db: &'a DbImpl<Store>, conditions: &'a Vec<QueryCondition>) -> Self {
        Self {
            offset,
            counter: 0,
            db,
            conditions,
        }
    }
}

impl<'a, Store: StorageData> LimitOffsetHandler<'a, Store> {
    pub fn new(
        limit: u64,
        offset: u64,
        db: &'a DbImpl<Store>,
        conditions: &'a Vec<QueryCondition>,
    ) -> Self {
        Self {
            limit: limit + offset,
            offset,
            counter: 0,
            db,
            conditions,
        }
    }
}

impl<'a, Store: StorageData> PathHandler<'a, Store> {
    pub fn new(db: &'a DbImpl<Store>, conditions: &'a Vec<QueryCondition>) -> Self {
        Self { db, conditions }
    }
}

impl<'a, Store: StorageData> SearchHandler for DefaultHandler<'a, Store> {
    fn process(&mut self, index: GraphIndex, distance: u64) -> Result<SearchControl, DbError> {
        self.db
            .evaluate_conditions(index, distance, self.conditions)
    }
}

impl<'a, Store: StorageData> SearchHandler for LimitHandler<'a, Store> {
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

impl<'a, Store: StorageData> SearchHandler for OffsetHandler<'a, Store> {
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

impl<'a, Store: StorageData> SearchHandler for LimitOffsetHandler<'a, Store> {
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

impl<'a, Store: StorageData> PathSearchHandler for PathHandler<'a, Store> {
    fn process(&self, index: GraphIndex, distance: u64) -> Result<(u64, bool), DbError> {
        match self
            .db
            .evaluate_conditions(index, distance, self.conditions)?
        {
            SearchControl::Continue(add) => Ok((1 + ((!add) as u64), add)),
            SearchControl::Finish(add) | SearchControl::Stop(add) => Ok((0, add)),
        }
    }
}
