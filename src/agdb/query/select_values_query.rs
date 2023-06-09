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
        let db_ids = match &self.ids {
            QueryIds::Ids(ids) => {
                let mut db_ids = vec![];
                db_ids.reserve(ids.len());

                for query_id in ids {
                    db_ids.push(db.db_id(query_id)?);
                }

                db_ids
            }
            QueryIds::Search(search_query) => search_query.search(db)?,
        };

        result.elements.reserve(db_ids.len());
        result.result = db_ids.len() as i64;

        for db_id in db_ids {
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
}
