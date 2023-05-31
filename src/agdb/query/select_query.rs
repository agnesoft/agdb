use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::Query;
use crate::Db;
use crate::DbElement;
use crate::QueryError;
use crate::QueryResult;

pub struct SelectQuery(pub QueryIds);

impl Query for SelectQuery {
    fn process(&self, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        match &self.0 {
            QueryIds::Ids(ids) => {
                for id in ids {
                    Self::select_id(id, db, result)?
                }
                Ok(())
            }
            QueryIds::Search(_) => Err(QueryError::from("Invalid select query")),
        }
    }
}

impl SelectQuery {
    fn select_id(query_id: &QueryId, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        let db_id = db.db_id(query_id)?;

        result.result += 1;
        result.elements.push(DbElement {
            index: db_id,
            values: db.values(db_id)?,
        });

        Ok(())
    }
}
