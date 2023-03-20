use crate::db::db_data::DbData;
use crate::query::Query;
use crate::query::QueryMut;
use crate::QueryError;
use crate::QueryResult;

pub struct Transaction<'a> {
    pub(crate) data: &'a DbData,
}

impl<'a> Transaction<'a> {
    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }

    pub(crate) fn commit(self) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }

    pub(crate) fn rollback(self) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }
}

pub struct TransactionMut<'a> {
    pub(crate) data: &'a mut DbData,
}

impl<'a> TransactionMut<'a> {
    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }

    pub fn exec_mut<T: QueryMut>(&self, query: &T) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }

    pub(crate) fn commit(self) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }

    pub(crate) fn rollback(self) -> Result<QueryResult, QueryError> {
        Ok(QueryResult::default())
    }
}
