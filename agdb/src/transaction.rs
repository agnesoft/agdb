use crate::query::Query;
use crate::storage::StorageData;
use crate::DbImpl;
use crate::QueryError;
use crate::QueryResult;

/// The `Transaction` is a proxy struct that
/// encapsulates an immutably borrowed `Db`.
/// It allows running queries via `exec()`.
pub struct Transaction<'a, Store: StorageData> {
    db: &'a DbImpl<Store>,
}

impl<'a, Store: StorageData> Transaction<'a, Store> {
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
        query.process(self.db)
    }

    pub(crate) fn new(data: &'a DbImpl<Store>) -> Self {
        Self { db: data }
    }
}
