use crate::{Query, QueryResult};

#[derive(Default)]
pub struct Transaction {}

impl Transaction {
    pub fn exec(&self, _query: Query) -> QueryResult {
        QueryResult::default()
    }

    pub fn transaction(&self) -> Transaction {
        Transaction::default()
    }
}
