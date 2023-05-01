pub mod db_element;
pub mod db_error;
pub mod db_id;
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
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;

pub struct Db {
    pub(crate) graph: Graph,
    pub(crate) aliases: IndexedMap<String, DbId>,
    pub(crate) indexes: IndexedMap<DbId, GraphIndex>,
    pub(crate) next_id: i64,
}

impl Db {
    pub fn new(_filename: &str) -> Result<Db, DbError> {
        Ok(Self {
            graph: Graph::new(),
            aliases: IndexedMap::<String, DbId>::new(),
            indexes: IndexedMap::<DbId, GraphIndex>::new(),
            next_id: 1,
        })
    }

    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        self.transaction(|transaction| transaction.exec(query))
    }

    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError> {
        self.transaction_mut(|transaction| transaction.exec_mut(query))
    }

    pub fn transaction<T, E>(&self, f: impl Fn(&Transaction) -> Result<T, E>) -> Result<T, E> {
        let transaction = Transaction::new(self);

        f(&transaction)
    }

    pub fn transaction_mut<T, E: From<QueryError>>(
        &mut self,
        f: impl Fn(&mut TransactionMut) -> Result<T, E>,
    ) -> Result<T, E> {
        let mut transaction = TransactionMut::new(&mut *self);
        let result = f(&mut transaction);
        Self::finish_transaction(transaction, result.is_ok())?;
        result
    }

    pub(crate) fn graph_index_from_id(&self, id: &QueryId) -> Result<GraphIndex, QueryError> {
        let db_id = match id {
            QueryId::Id(id) => *id,
            QueryId::Alias(alias) => self
                .aliases
                .value(alias)?
                .ok_or(QueryError::from(format!("Alias '{alias}' not found")))?,
        };

        self.indexes
            .value(&db_id)?
            .ok_or(QueryError::from(format!("Id '{}' not found", db_id.0)))
    }

    fn finish_transaction(transaction: TransactionMut, result: bool) -> Result<(), QueryError> {
        if result {
            transaction.commit()
        } else {
            transaction.rollback()
        }
    }
}
