use super::query_ids::QueryIds;
use crate::Db;
use crate::DbElement;
use crate::DbKey;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;

pub struct SelectValuesQuery {
    pub keys: Vec<DbKey>,
    pub ids: QueryIds,
}

impl Query for SelectValuesQuery {
    fn process(&self, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        match &self.ids {
            QueryIds::Ids(ids) => {
                result.elements.reserve(ids.len());
                result.result = ids.len() as i64;

                for id in ids {
                    let db_id = db.db_id(id)?;
                    let values = db.values_by_keys(db_id, &self.keys)?;

                    if values.len() != self.keys.len() {
                        for key in &self.keys {
                            if !values.iter().any(|x| x.key == *key) {
                                return Err(QueryError::from(format!(
                                    "Missing key '{}' for id '{}'",
                                    key, db_id.0
                                )));
                            }
                        }
                    }

                    result.elements.push(DbElement { id: db_id, values });
                }

                Ok(())
            }
            QueryIds::Search(_) => Err(QueryError::from("Invalid select values query")),
        }
    }
}
