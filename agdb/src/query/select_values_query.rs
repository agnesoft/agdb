use crate::DbElement;
use crate::DbImpl;
use crate::DbKey;
use crate::Query;
use crate::QueryError;
use crate::QueryIds;
use crate::QueryResult;
use crate::StorageData;

/// Query to select elements with only certain properties of
/// given ids. All ids must exist in the database and all
/// of them must have the requested properties.
///
/// The result will be number of elements and the
/// list of elements with the requested properties.
pub struct SelectValuesQuery {
    pub keys: Vec<DbKey>,
    pub ids: QueryIds,
}

impl Query for SelectValuesQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        let (db_ids, is_search) = match &self.ids {
            QueryIds::Ids(ids) => {
                let mut db_ids = Vec::with_capacity(ids.len());

                for query_id in ids {
                    db_ids.push(db.db_id(query_id)?);
                }

                (db_ids, false)
            }
            QueryIds::Search(search_query) => (search_query.search(db)?, true),
        };

        result.elements.reserve(db_ids.len());
        result.result = db_ids.len() as i64;

        for id in db_ids {
            let values = db.values_by_keys(id, &self.keys)?;

            if !is_search && values.len() != self.keys.len() {
                for key in &self.keys {
                    if !values.iter().any(|x| x.key == *key) {
                        return Err(QueryError::from(format!(
                            "Missing key '{}' for id '{}'",
                            key, id.0
                        )));
                    }
                }
            }

            result.elements.push(DbElement {
                id,
                from: db.from_id(id),
                to: db.to_id(id),
                values,
            });
        }

        Ok(result)
    }
}
