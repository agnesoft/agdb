use crate::DbError;
use crate::DbImpl;
use crate::Query;
use crate::QueryResult;
use crate::StorageData;

/// Query to select number of edges in the database.
///
/// The edge count is returned in `QueryResult::result`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[derive(Clone, Debug, PartialEq)]
pub struct SelectAllEdgeCountQuery {}

impl Query for SelectAllEdgeCountQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError> {
        Ok(QueryResult {
            result: db.all_edge_count()?,
            elements: vec![],
        })
    }
}

impl Query for &SelectAllEdgeCountQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError> {
        (*self).process(db)
    }
}
