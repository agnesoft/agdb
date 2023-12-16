use crate::query::query_values::QueryValues;
use crate::DbImpl;
use crate::QueryError;
use crate::QueryIds;
use crate::QueryMut;
use crate::QueryResult;
use crate::StorageData;

/// Query to insert or update key-value pairs (properties)
/// to existing elements in the database. All `ids` must exist
/// in the database. If `values` is set to `Single` the properties
/// will be inserted uniformly to all `ids` otherwise there must be
/// enough `values` for all `ids`.
///
/// The result will be number of inserted/update values and no elements.
///
/// NOTE: The result is NOT number of affected elements but individual properties.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InsertValuesQuery {
    /// Ids whose properties should be updated
    pub ids: QueryIds,

    /// Key value pairs to be inserted to the existing elements.
    pub values: QueryValues,
}

impl QueryMut for InsertValuesQuery {
    fn process<Store: StorageData>(
        &self,
        db: &mut DbImpl<Store>,
    ) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        match &self.ids {
            QueryIds::Ids(ids) => match &self.values {
                QueryValues::Single(values) => {
                    for id in ids {
                        let db_id = db.db_id(id)?;

                        for key_value in values {
                            db.insert_or_replace_key_value(db_id, key_value)?;
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
                            db.insert_or_replace_key_value(db_id, key_value)?;
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
                                db.insert_or_replace_key_value(db_id, key_value)?;
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
                                db.insert_or_replace_key_value(*db_id, key_value)?;
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
