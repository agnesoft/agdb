use super::QueryMut;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

pub struct RemoveAliasesQuery(pub Vec<String>);

impl QueryMut for RemoveAliasesQuery {
    fn process(&self, db: &mut Db) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        for alias in &self.0 {
            if db.remove_alias(alias)? {
                result.result -= 1;
            }
        }

        Ok(result)
    }
}
