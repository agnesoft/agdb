pub mod db_element;
pub mod db_error;
pub mod db_index;
pub mod db_key;
pub mod db_key_value;
pub mod db_value;

mod db_float;

use crate::query::Query;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;

#[derive(Default)]
pub struct Db {}

impl Db {
    pub fn exec<T: Query>(&self, _query: T) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }

    pub fn transaction(&self) -> Transaction {
        Transaction::default()
    }
}
