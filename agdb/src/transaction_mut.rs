use crate::DbError;
use crate::DbImpl;
use crate::QueryResult;
use crate::StorageData;
use crate::Transaction;
use crate::query::Query;
use crate::query::QueryMut;

/// The `TransactionMut` is a proxy struct that
/// encapsulates a mutably borrowed [`DbImpl`].
/// It allows running queries via [`exec()`](#method.exec) and [`exec_mut()`](#method.exec_mut).
pub struct TransactionMut<'a, Store: StorageData> {
    db: &'a mut DbImpl<Store>,
}

impl<'a, Store: StorageData> TransactionMut<'a, Store> {
    /// Executes immutable queries:
    ///
    /// - Select elements
    /// - Select values
    /// - Select keys
    /// - Select key count
    /// - Select aliases
    /// - Select all aliases
    /// - Search
    pub fn exec<T: Query>(&self, query: T) -> Result<QueryResult, DbError> {
        Transaction::new(self.db).exec(query)
    }

    /// Executes mutable queries:
    ///
    /// - Insert nodes
    /// - Insert edges
    /// - Insert aliases
    /// - Insert values
    /// - Remove elements
    /// - Remove aliases
    /// - Remove values
    pub fn exec_mut<T: QueryMut>(&mut self, query: T) -> Result<QueryResult, DbError> {
        query.process(self.db)
    }

    pub(crate) fn new(data: &'a mut DbImpl<Store>) -> Self {
        Self { db: data }
    }

    pub(crate) fn commit(self) -> Result<(), DbError> {
        self.db.commit()?;
        Ok(())
    }

    pub(crate) fn rollback(self) -> Result<(), DbError> {
        self.db.rollback()?;
        Ok(())
    }
}
