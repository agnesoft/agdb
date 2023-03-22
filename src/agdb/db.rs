pub mod db_context;
pub mod db_data;
pub mod db_element;
pub mod db_error;
pub mod db_index;
pub mod db_key;
pub mod db_key_value;
pub mod db_value;

mod db_float;

use self::db_data::DbData;
use crate::query::Query;
use crate::query::QueryMut;
use crate::transaction_mut::TransactionMut;
use crate::DbError;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;
use std::sync::RwLock;

pub struct Db {
    data: RwLock<DbData>,
}

impl Db {
    pub fn new(_filename: &str) -> Result<Db, DbError> {
        Ok(Self {
            data: RwLock::new(DbData::new()?),
        })
    }

    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        self.transaction(|transaction| transaction.exec(query))
    }

    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError> {
        self.transaction_mut(|transaction| transaction.exec_mut(query))
    }

    pub fn transaction<T, E>(&self, f: impl Fn(&Transaction) -> Result<T, E>) -> Result<T, E> {
        let transaction = Transaction {
            data: &self.data.read().unwrap(),
        };

        f(&transaction)
    }

    pub fn transaction_mut<T, E: From<QueryError>>(
        &mut self,
        f: impl Fn(&mut TransactionMut) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut data = self
            .data
            .write()
            .map_err(|err| QueryError::from(err.to_string()))?;
        let mut transaction = TransactionMut::new(&mut data);

        let result = f(&mut transaction);

        if result.is_ok() {
            transaction.commit()?;
        } else {
            transaction.rollback()?;
        }

        result
    }
}
