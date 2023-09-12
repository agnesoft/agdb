use crate::db::DbImpl;
use crate::storage::StorageData;
use crate::QueryError;
use crate::QueryMut;
use crate::QueryResult;

/// Query to remove aliases from the database. It
/// is not an error if an alias to be removed already
/// does not exist.
///
/// The result will be a negative number signifying how
/// many aliases have been actually removed.
pub struct RemoveAliasesQuery(pub Vec<String>);

impl QueryMut for RemoveAliasesQuery {
    fn process<Store: StorageData>(
        &self,
        db: &mut DbImpl<Store>,
    ) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        for alias in &self.0 {
            if db.remove_alias(alias)? {
                result.result -= 1;
            }
        }

        Ok(result)
    }
}
