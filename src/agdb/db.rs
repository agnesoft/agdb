pub mod db_context;
pub mod db_element;
pub mod db_error;
pub mod db_index;
pub mod db_key;
pub mod db_key_value;
pub mod db_value;

mod db_float;

use crate::collections::indexed_map::IndexedMap;
use crate::graph::graph_index::GraphIndex;
use crate::graph::Graph;
use crate::query::query_id::QueryId;
use crate::query::Query;
use crate::query::QueryMut;
use crate::transaction_mut::TransactionMut;
use crate::DbError;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;

pub struct Db {
    pub(crate) graph: Graph,
    pub(crate) aliases: IndexedMap<String, i64>,
    pub(crate) indexes: IndexedMap<i64, i64>,
    pub(crate) next_node: i64,
    pub(crate) next_edge: i64,
}

impl Db {
    pub fn new(_filename: &str) -> Result<Db, DbError> {
        Ok(Self {
            graph: Graph::new(),
            aliases: IndexedMap::<String, i64>::new(),
            indexes: IndexedMap::<i64, i64>::new(),
            next_node: 1,
            next_edge: -1,
        })
    }

    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        self.transaction(|transaction| transaction.exec(query))
    }

    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError> {
        self.transaction_mut(|transaction| transaction.exec_mut(query))
    }

    pub fn transaction<T, E>(&self, f: impl Fn(&Transaction) -> Result<T, E>) -> Result<T, E> {
        let transaction = Transaction::new(&self);

        f(&transaction)
    }

    pub fn transaction_mut<T, E: From<QueryError>>(
        &mut self,
        f: impl Fn(&mut TransactionMut) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut transaction = TransactionMut::new(&mut *self);

        let result = f(&mut transaction);

        if result.is_ok() {
            transaction.commit()?;
        } else {
            transaction.rollback()?;
        }

        result
    }

    pub(crate) fn graph_index_from_id(&self, id: &QueryId) -> Result<GraphIndex, QueryError> {
        Ok(GraphIndex::from(self.index_from_id(id)?))
    }

    pub(crate) fn index_from_id(&self, id: &QueryId) -> Result<i64, QueryError> {
        Ok(match id {
            QueryId::Id(id) => self
                .indexes
                .value(id)?
                .ok_or(QueryError::from(format!("Id '{id}' not found")))?,
            QueryId::Alias(alias) => self
                .aliases
                .value(alias)?
                .ok_or(QueryError::from(format!("Alias '{alias}' not found")))?,
        })
    }
}
