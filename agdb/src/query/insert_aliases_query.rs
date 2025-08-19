use crate::DbError;
use crate::DbImpl;
use crate::QueryIds;
use crate::QueryMut;
use crate::QueryResult;
use crate::StorageData;

/// Query to insert or update aliases of existing nodes.
/// All `ids` must exist. None of the `aliases` can be empty.
/// If there is an existing alias for any of the elements it
/// will be overwritten with a new one.
///
/// NOTE: Setting `ids` to a search query will result in an error.
///
/// The result will contain number of aliases inserted/updated but no elements.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
#[derive(Clone, Debug, PartialEq)]
pub struct InsertAliasesQuery {
    /// Ids to be aliased
    pub ids: QueryIds,

    /// Aliases to be inserted
    pub aliases: Vec<String>,
}

impl QueryMut for InsertAliasesQuery {
    fn process<Store: StorageData>(&self, db: &mut DbImpl<Store>) -> Result<QueryResult, DbError> {
        let mut result = QueryResult::default();

        match &self.ids {
            QueryIds::Ids(ids) => {
                if ids.len() != self.aliases.len() {
                    return Err(DbError::from("Ids and aliases must have the same length"));
                }

                for (id, alias) in ids.iter().zip(&self.aliases) {
                    if alias.is_empty() {
                        return Err(DbError::from("Empty alias is not allowed"));
                    }

                    let db_id = db.db_id(id)?;
                    db.insert_alias(db_id, alias)?;
                    result.result += 1;
                }
            }
            QueryIds::Search(_) => {
                return Err(DbError::from(
                    "Insert aliases query does not support search queries",
                ));
            }
        }

        Ok(result)
    }
}

impl QueryMut for &InsertAliasesQuery {
    fn process<Store: StorageData>(&self, db: &mut DbImpl<Store>) -> Result<QueryResult, DbError> {
        (*self).process(db)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Db;
    use crate::DbId;
    use crate::SearchQueryAlgorithm;
    use crate::query::query_id::QueryId;
    use crate::query::search_query::SearchQuery;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn invalid_query() {
        let test_file = TestFile::new();
        let mut db = Db::new(test_file.file_name()).unwrap();
        let query = InsertAliasesQuery {
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::BreadthFirst,
                origin: QueryId::Id(DbId(0)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
            }),
            aliases: vec![],
        };
        assert_eq!(
            query.process(&mut db).unwrap_err(),
            DbError::from("Insert aliases query does not support search queries")
        );
    }
}
