use crate::query::Query;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

/// The `Transaction` is a proxy struct that
/// encapsulates an immutably borrowed `Db`.
/// It allows running queries via `exec()`.
pub struct Transaction<'a> {
    db: &'a Db,
}

impl<'a> Transaction<'a> {
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
        let mut result = QueryResult {
            result: 0,
            elements: vec![],
        };

        query.process(self.db, &mut result)?;

        Ok(result)
    }

    pub(crate) fn new(data: &'a Db) -> Self {
        Self { db: data }
    }
}
