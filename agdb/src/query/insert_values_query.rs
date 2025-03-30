use crate::DbElement;
use crate::DbId;
use crate::DbImpl;
use crate::DbKeyValue;
use crate::QueryError;
use crate::QueryId;
use crate::QueryIds;
use crate::QueryMut;
use crate::QueryResult;
use crate::SearchQuery;
use crate::StorageData;
use crate::query::query_values::QueryValues;
use crate::query_builder::search::SearchQueryBuilder;

/// Query to insert or update key-value pairs (properties)
/// to existing elements in the database. All `ids` must exist
/// in the database. If `values` is set to `Single` the properties
/// will be inserted uniformly to all `ids` otherwise there must be
/// enough `values` for all `ids`.
///
/// The result will be number of inserted/updated values and inserted new
/// elements (nodes).
///
/// NOTE: The result is NOT number of affected elements but individual properties.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::AgdbDeSerialize))]
#[cfg_attr(feature = "api", derive(agdb::api::ApiDef))]
#[derive(Clone, Debug, PartialEq)]
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
                        insert_values(db, id, values, &mut result)?;
                    }
                }
                QueryValues::Multi(values) => {
                    if ids.len() != values.len() {
                        return Err(QueryError::from("Ids and values length do not match"));
                    }

                    for (id, values) in ids.iter().zip(values) {
                        insert_values(db, id, values, &mut result)?;
                    }
                }
            },
            QueryIds::Search(search_query) => {
                let db_ids = search_query.search(db)?;

                match &self.values {
                    QueryValues::Single(values) => {
                        for db_id in db_ids {
                            insert_values_id(db, db_id, values, &mut result)?;
                        }
                    }
                    QueryValues::Multi(values) => {
                        if db_ids.len() != values.len() {
                            return Err(QueryError::from("Ids and values length do not match"));
                        }

                        for (db_id, values) in db_ids.iter().zip(values) {
                            insert_values_id(db, *db_id, values, &mut result)?;
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}

impl QueryMut for &InsertValuesQuery {
    fn process<Store: StorageData>(
        &self,
        db: &mut DbImpl<Store>,
    ) -> Result<QueryResult, QueryError> {
        (*self).process(db)
    }
}

fn insert_values<Store: StorageData>(
    db: &mut DbImpl<Store>,
    id: &QueryId,
    values: &[DbKeyValue],
    result: &mut QueryResult,
) -> Result<(), QueryError> {
    match db.db_id(id) {
        Ok(db_id) => insert_values_id(db, db_id, values, result),
        Err(e) => match id {
            QueryId::Id(id) => {
                if id.0 == 0 {
                    insert_values_new(db, None, values, result)
                } else {
                    Err(e)
                }
            }
            QueryId::Alias(alias) => insert_values_new(db, Some(alias), values, result),
        },
    }
}

fn insert_values_new<Store: StorageData>(
    db: &mut DbImpl<Store>,
    alias: Option<&String>,
    values: &[DbKeyValue],
    result: &mut QueryResult,
) -> Result<(), QueryError> {
    let db_id = db.insert_node()?;

    if let Some(alias) = alias {
        db.insert_new_alias(db_id, alias)?;
    }

    for key_value in values {
        db.insert_key_value(db_id, key_value)?;
    }

    result.result += values.len() as i64;
    result.elements.push(DbElement {
        id: db_id,
        from: None,
        to: None,
        values: vec![],
    });

    Ok(())
}

fn insert_values_id<Store: StorageData>(
    db: &mut DbImpl<Store>,
    db_id: DbId,
    values: &[DbKeyValue],
    result: &mut QueryResult,
) -> Result<(), QueryError> {
    for key_value in values {
        db.insert_or_replace_key_value(db_id, key_value)?;
        result.result += 1;
    }
    Ok(())
}

impl SearchQueryBuilder for InsertValuesQuery {
    fn search_mut(&mut self) -> &mut SearchQuery {
        if let QueryIds::Search(search) = &mut self.ids {
            search
        } else {
            panic!("Expected search query");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn missing_search() {
        InsertValuesQuery {
            values: QueryValues::Single(vec![]),
            ids: QueryIds::Ids(vec![]),
        }
        .search_mut();
    }
}
