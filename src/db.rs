use crate::{Query, QueryResult};

#[derive(Default)]
pub struct Db {}

impl Db {
    pub fn exec(&self, _query: Query) -> QueryResult {
        QueryResult::default()
    }
}
