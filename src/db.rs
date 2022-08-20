use crate::{Query, QueryError, QueryResult, Transaction};

#[derive(Default)]
pub struct Db {}

impl Db {
    pub fn exec(&self, _query: Query) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }

    pub fn transaction(&self) -> Transaction {
        Transaction::default()
    }
}
