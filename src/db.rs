use crate::{Query, QueryResult, Transaction};

#[derive(Default)]
pub struct Db {}

impl Db {
    pub fn exec(&self, _query: Query) -> QueryResult {
        QueryResult::default()
    }

    pub fn transaction(&self) -> Transaction {
        Transaction::default()
    }
}
