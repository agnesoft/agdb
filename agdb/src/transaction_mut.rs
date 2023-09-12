use crate::query::Query;
use crate::query::QueryMut;
use crate::DbImpl;
use crate::QueryError;
use crate::QueryResult;
use crate::StorageData;
use crate::Transaction;

/// The `TransactionMut` is a proxy struct that
/// encapsulates a mutably borrowed `Db`.
/// It allows running queries via `exec()` and `exec_mut()`.
pub struct TransactionMut<'a, Store: StorageData> {
    db: &'a mut DbImpl<Store>,
}

impl<'a, Store: StorageData> TransactionMut<'a, Store> {
    /// Executes immutable query:
    ///
    /// - Select elements
    /// - Select values
    /// - Select keys
    /// - Select key count
    /// - Select aliases
    /// - Select all aliases
    /// - Search
    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        Transaction::new(self.db).exec(query)
    }

    /// Executes mutable query:
    ///
    /// - Insert nodes
    /// - Insert edges
    /// - Insert aliases
    /// - Insert values
    /// - Remove elements
    /// - Remove aliases
    /// - Remove values
    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError> {
        query.process(self.db)
    }

    pub(crate) fn new(data: &'a mut DbImpl<Store>) -> Self {
        Self { db: data }
    }

    pub(crate) fn commit(self) -> Result<(), QueryError> {
        self.db.commit()?;
        Ok(())
    }

    pub(crate) fn rollback(self) -> Result<(), QueryError> {
        self.db.rollback()?;
        Ok(())
    }
}
