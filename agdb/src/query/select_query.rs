use crate::DbElement;
use crate::DbImpl;
use crate::Query;
use crate::QueryError;
use crate::QueryIds;
use crate::QueryResult;
use crate::StorageData;

/// Query to select elements with all properties of
/// given ids. All ids must exist in the database.
///
/// The result will be number of elements and the
/// list of elements with all properties.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct SelectQuery(pub QueryIds);

impl Query for SelectQuery {
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

        for id in db_ids {
            result.elements.push(DbElement {
                id,
                from: db.from_id(id),
                to: db.to_id(id),
                values: db.values(id)?,
            });
        }

        Ok(result)
    }
}
