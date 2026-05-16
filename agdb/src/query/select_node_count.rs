use crate::DbError;
use crate::DbImpl;
use crate::Query;
use crate::QueryResult;
use crate::StorageData;

/// Query to select number of nodes in the database.
///
/// The node count is returned in `QueryResult::result`.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
#[derive(Clone, Debug, PartialEq)]
pub struct SelectNodeCountQuery {}

impl Query for SelectNodeCountQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError> {
        Ok(QueryResult {
            result: db.node_count()?,
            elements: vec![],
        })
    }
}

impl Query for &SelectNodeCountQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError> {
        (*self).process(db)
    }
}
