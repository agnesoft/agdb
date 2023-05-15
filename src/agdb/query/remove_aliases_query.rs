use super::QueryMut;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

pub struct RemoveAliasesQuery {
    pub aliases: Vec<String>,
}

impl QueryMut for RemoveAliasesQuery {
    fn process(&self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        for alias in &self.aliases {
            if db.remove_alias(alias)? {
                result.result -= 1;
            }
        }

        Ok(())
    }
}
