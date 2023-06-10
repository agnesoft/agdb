use super::query_ids::QueryIds;
use super::select_values_query::SelectValuesQuery;
use crate::Db;
use crate::QueryError;
use crate::QueryMut;
use crate::QueryResult;

pub struct RemoveValuesQuery(pub SelectValuesQuery);

impl QueryMut for RemoveValuesQuery {
    fn process(&self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        match &self.0.ids {
            QueryIds::Ids(ids) => {
                for id in ids {
                    let db_id = db.db_id(id)?;
                    result.result += db.remove_keys(db_id, &self.0.keys)?;
                }
            }
            QueryIds::Search(search_query) => {
                for db_id in search_query.search(db)? {
                    result.result += db.remove_keys(db_id, &self.0.keys)?;
                }
            }
        }

        Ok(())
    }
}
