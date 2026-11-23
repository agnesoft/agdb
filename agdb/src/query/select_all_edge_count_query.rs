use crate::DbError;
use crate::Query;
use crate::QueryResult;
use crate::StorageData;
use crate::Transaction;

/// Query to select number of edges in the database.
///
/// The edge count is returned in `QueryResult::result`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct SelectAllEdgeCountQuery {}

impl Query for SelectAllEdgeCountQuery {
    fn process<Store: StorageData>(&self, db: &Transaction<Store>) -> Result<QueryResult, DbError> {
        Ok(QueryResult {
            result: db.all_edge_count(),
            elements: vec![],
        })
    }
}

impl Query for &SelectAllEdgeCountQuery {
    fn process<Store: StorageData>(&self, db: &Transaction<Store>) -> Result<QueryResult, DbError> {
        (*self).process(db)
    }
}
