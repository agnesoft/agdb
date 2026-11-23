use crate::DbError;
use crate::DbValue;
use crate::QueryMut;
use crate::QueryResult;
use crate::StorageData;
use crate::TransactionMut;

/// Query to remove an existing index on
/// a given key.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct RemoveIndexQuery(pub DbValue);

impl QueryMut for RemoveIndexQuery {
    fn process<Store: StorageData>(
        &self,
        db: &mut TransactionMut<Store>,
    ) -> Result<QueryResult, DbError> {
        let result = db.remove_index(&self.0)?;

        Ok(QueryResult {
            result,
            elements: vec![],
        })
    }
}

impl QueryMut for &RemoveIndexQuery {
    fn process<Store: StorageData>(
        &self,
        db: &mut TransactionMut<Store>,
    ) -> Result<QueryResult, DbError> {
        (*self).process(db)
    }
}
