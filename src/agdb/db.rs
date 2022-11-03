pub mod db_error;
pub mod db_value;

use crate::Query;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;

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
