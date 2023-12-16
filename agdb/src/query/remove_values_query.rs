use crate::DbImpl;
use crate::QueryError;
use crate::QueryIds;
use crate::QueryMut;
use crate::QueryResult;
use crate::SelectValuesQuery;
use crate::StorageData;

/// Query to remove properties from existing elements
/// in the database. All of the specified `ids` must
/// exist in the database however they do not need to have
/// all the listed keys (it is NOT an error if any or all keys
/// do not exist on any of the elements).
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RemoveValuesQuery(pub SelectValuesQuery);

impl QueryMut for RemoveValuesQuery {
    fn process<Store: StorageData>(
        &self,
        db: &mut DbImpl<Store>,
    ) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        match &self.0.ids {
            QueryIds::Ids(ids) => {
                for id in ids {
                    let db_id = db.db_id(id)?;
                    result.result += db.remove_keys(db_id, &self.0.keys)?;
                }
            }
            QueryIds::Search(search_query) => {
                for db_id in search_query.search(db)? {
                    result.result += db.remove_keys(db_id, &self.0.keys)?;
                }
            }
        }

        Ok(result)
    }
}
