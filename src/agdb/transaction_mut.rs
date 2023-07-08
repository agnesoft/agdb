use crate::query::Query;
use crate::query::QueryMut;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;

pub struct TransactionMut<'a> {
    db: &'a mut Db,
}

impl<'a> TransactionMut<'a> {
    pub fn new(data: &'a mut Db) -> Self {
        Self { db: data }
    }

    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        Transaction::new(self.db).exec(query)
    }

    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError> {
        query.process(self.db)
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
