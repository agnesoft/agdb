use super::query_ids::QueryIds;
use super::QueryMut;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

pub struct RemoveQuery(pub QueryIds);

impl QueryMut for RemoveQuery {
    fn process(&self, db: &mut Db) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        match &self.0 {
            QueryIds::Ids(ids) => {
                for id in ids {
                    if db.remove(id)? {
                        result.result -= 1;
                    }
                }
            }
            QueryIds::Search(search_query) => {
                for db_id in search_query.search(db)? {
                    if db.remove_id(db_id)? {
                        result.result -= 1;
                    }
                }
            }
        }

        Ok(result)
    }
}
