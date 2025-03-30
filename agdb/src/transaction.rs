use crate::DbImpl;
use crate::QueryError;
use crate::QueryResult;
use crate::StorageData;
use crate::query::Query;

/// The `Transaction` is a proxy struct that
/// encapsulates an immutably borrowed [`DbImpl`].
/// It allows running queries via [`exec()`](#method.exec).
pub struct Transaction<'a, Store: StorageData> {
    db: &'a DbImpl<Store>,
}

impl<'a, Store: StorageData> Transaction<'a, Store> {
    /// Executes immutable queries:
    ///
    /// - Select elements
    /// - Select values
    /// - Select keys
    /// - Select key count
    /// - Select aliases
    /// - Select all aliases
    /// - Search
    pub fn exec<T: Query>(&self, query: T) -> Result<QueryResult, QueryError> {
        query.process(self.db)
    }

    pub(crate) fn new(data: &'a DbImpl<Store>) -> Self {
        Self { db: data }
    }
}
