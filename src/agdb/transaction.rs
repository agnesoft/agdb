use crate::query::Query;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

pub struct Transaction<'a> {
    db: &'a Db,
}

impl<'a> Transaction<'a> {
    pub fn new(data: &'a Db) -> Self {
        Self { db: data }
    }

    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        query.process(self.db)
    }
}
