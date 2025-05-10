use crate::DbElement;
use crate::DbId;
use crate::DbImpl;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;
use crate::StorageData;

/// Query to select all indexes in the database.
///
/// The result will be number of returned indexes and single element
/// with index 0 and the properties corresponding to the names of the indexes
/// (keys) with `u64` values representing number of indexed values in each
/// index.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::AgdbDeSerialize))]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
#[derive(Clone, Debug, PartialEq)]
pub struct SelectIndexesQuery {}

impl Query for SelectIndexesQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();
        let indexes = db.indexes();
        result.result = indexes.len() as i64;

        result.elements.push(DbElement {
            id: DbId(0),
            from: None,
            to: None,
            values: indexes,
        });

        Ok(result)
    }
}

impl Query for &SelectIndexesQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, QueryError> {
        (*self).process(db)
    }
}
