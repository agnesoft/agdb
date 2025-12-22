use crate::DbElement;
use crate::DbError;
use crate::DbId;
use crate::DbImpl;
use crate::Query;
use crate::QueryResult;
use crate::StorageData;

/// Query to select number of nodes in the database.
///
/// The result will be 1 and elements with a single element
/// of id 0 and a single property `String("node_count")` with
/// a value `u64` represneting number of nodes in teh database.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::TypeDefImpl))]
#[derive(Clone, Debug, PartialEq)]
pub struct SelectNodeCountQuery {}

impl Query for SelectNodeCountQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError> {
        Ok(QueryResult {
            result: 1,
            elements: vec![DbElement {
                id: DbId::default(),
                from: None,
                to: None,
                values: vec![("node_count", db.node_count()?).into()],
            }],
        })
    }
}

impl Query for &SelectNodeCountQuery {
    fn process<Store: StorageData>(&self, db: &DbImpl<Store>) -> Result<QueryResult, DbError> {
        (*self).process(db)
    }
}
