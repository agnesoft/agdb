use crate::DbElement;
use crate::DbImpl;
use crate::Query;
use crate::QueryError;
use crate::QueryIds;
use crate::QueryResult;
use crate::StorageData;

/// Query to select only property keys of given ids. All
/// of the ids must exist in the database.
///
/// The result will be number of elements returned and the list
/// of elements with all properties except all values will be empty.
pub struct SelectKeysQuery(pub QueryIds);

impl Query for SelectKeysQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        let db_ids = match &self.0 {
            QueryIds::Ids(ids) => {
                let mut db_ids = Vec::with_capacity(ids.len());

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
                values: db.keys(db_id)?,
            });
        }

        Ok(result)
    }
}
