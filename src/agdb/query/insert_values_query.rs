use super::query_ids::QueryIds;
use super::query_values::QueryValues;
use crate::Db;
use crate::QueryError;
use crate::QueryMut;
use crate::QueryResult;

pub struct InsertValuesQuery {
    pub ids: QueryIds,
    pub values: QueryValues,
}

impl QueryMut for InsertValuesQuery {
    fn process(&self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        if let QueryIds::Ids(ids) = &self.ids {
            if let QueryValues::Single(values) = &self.values {
                for id in ids {
                    let db_id = db.db_id(&id)?;
                    for key_value in values {
                        db.insert_key_value(&db_id, &key_value.key, &key_value.value)?;
                        result.result += 1;
                    }
                }

                return Ok(());
            } else if let QueryValues::Multi(values) = &self.values {
                if ids.len() != values.len() {
                    return Err(QueryError::from("Ids and values length do not match"));
                }

                for (id, values) in ids.iter().zip(values) {
                    let db_id = db.db_id(&id)?;
                    for key_value in values {
                        db.insert_key_value(&db_id, &key_value.key, &key_value.value)?;
                        result.result += 1;
                    }
                }

                return Ok(());
            }
        }

        Err(QueryError::from("Invalid insert aliases query"))
    }
}
