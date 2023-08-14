use super::query_ids::QueryIds;
use super::QueryMut;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

/// Query to remove database elements (nodes & edges). It
/// is not an error if any of the `ids` do not already exist.
///
/// All properties associated with a given element are also removed.
///
/// If removing nodes all of its incoming and outgoing edges are
/// also removed along with their properties.
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