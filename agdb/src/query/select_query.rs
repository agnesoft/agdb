use super::query_ids::QueryIds;
use crate::storage::StorageData;
use crate::DbElement;
use crate::DbImpl;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;

/// Query to select elements with all properties of
/// given ids. All ids must exist in the database.
///
/// The result will be number of elements and the
/// list of elements with all properties.
pub struct SelectQuery(pub QueryIds);

impl Query for SelectQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        let db_ids = match &self.0 {
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
            result.elements.push(DbElement {
                id: db_id,
                values: db.values(db_id)?,
            });
        }

        Ok(result)
    }
}
