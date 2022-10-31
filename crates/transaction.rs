use crate::{Query, QueryError, QueryResult};

#[derive(Default)]
pub struct Transaction {}

impl Transaction {
    pub fn commit(self) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }

    pub fn exec(&self, _query: Query) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }

    pub fn rollback(self) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }

    pub fn transaction(&self) -> Transaction {
        Transaction::default()
    }
}
