use super::query_ids::QueryIds;
use crate::Db;
use crate::DbElement;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;

pub struct SelectQuery(pub QueryIds);

impl Query for SelectQuery {
    fn process(&self, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        match &self.0 {
            QueryIds::Ids(ids) => {
                result.elements.reserve(ids.len());
                result.result = ids.len() as i64;

                for id in ids {
                    let db_id = db.db_id(id)?;

                    result.elements.push(DbElement {
                        id: db_id,
                        values: db.values(db_id)?,
                    });
                }
                Ok(())
            }
            QueryIds::Search(_) => Err(QueryError::from("Invalid select query")),
        }
    }
}
