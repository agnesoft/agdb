use super::query_ids::QueryIds;
use super::QueryMut;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

pub struct RemoveQuery(pub QueryIds);

impl QueryMut for RemoveQuery {
    fn process(&self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        match &self.0 {
            QueryIds::Ids(ids) => {
                for id in ids {
                    if db.remove(id)? {
                        result.result -= 1;
                    }
                }

                Ok(())
            }
            QueryIds::Search(_) => Err(QueryError::from("Invalid remove query")),
        }
    }
}
