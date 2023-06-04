use super::query_ids::QueryIds;
use crate::Db;
use crate::DbElement;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;

pub struct SelectKeysQuery(pub QueryIds);

impl Query for SelectKeysQuery {
    fn process(&self, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        match &self.0 {
            QueryIds::Ids(ids) => {
                result.elements.reserve(ids.len());
                result.result += ids.len() as i64;

                for id in ids {
                    let db_id = db.db_id(id)?;
                    result.elements.push(DbElement {
                        index: db_id,
                        values: db.keys(db_id)?,
                    });
                }

                Ok(())
            }
            QueryIds::Search(_) => Err(QueryError::from("Invalid select keys query")),
        }
    }
}
