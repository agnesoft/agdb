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
    fn process(&self, db: &mut Db) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        match &self.ids {
            QueryIds::Ids(ids) => match &self.values {
                QueryValues::Single(values) => {
                    for id in ids {
                        let db_id = db.db_id(id)?;
                        for key_value in values {
                            db.insert_key_value(db_id, key_value)?;
                            result.result += 1;
                        }
                    }
                }
                QueryValues::Multi(values) => {
                    if ids.len() != values.len() {
                        return Err(QueryError::from("Ids and values length do not match"));
                    }

                    for (id, values) in ids.iter().zip(values) {
                        let db_id = db.db_id(id)?;
                        for key_value in values {
                            db.insert_key_value(db_id, key_value)?;
                            result.result += 1;
                        }
                    }
                }
            },
            QueryIds::Search(search_query) => {
                let db_ids = search_query.search(db)?;

                match &self.values {
                    QueryValues::Single(values) => {
                        for db_id in db_ids {
                            for key_value in values {
                                db.insert_key_value(db_id, key_value)?;
                                result.result += 1;
                            }
                        }
                    }
                    QueryValues::Multi(values) => {
                        if db_ids.len() != values.len() {
                            return Err(QueryError::from("Ids and values length do not match"));
                        }

                        for (db_id, values) in db_ids.iter().zip(values) {
                            for key_value in values {
                                db.insert_key_value(*db_id, key_value)?;
                                result.result += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}
