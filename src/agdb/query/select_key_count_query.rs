use super::query_ids::QueryIds;
use crate::Db;
use crate::DbElement;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;

pub struct SelectKeyCountQuery(pub QueryIds);

impl Query for SelectKeyCountQuery {
    fn process(&self, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        match &self.0 {
            QueryIds::Ids(ids) => {
                result.elements.reserve(ids.len());
                result.result += ids.len() as i64;

                for id in ids {
                    let db_id = db.db_id(id)?;
                    result.elements.push(DbElement {
                        index: db_id,
                        values: vec![("key_count", db.key_count(db_id)?).into()],
                    });
                }

                Ok(())
            }
            QueryIds::Search(_) => Err(QueryError::from("Invalid select key count query")),
        }
    }
}
