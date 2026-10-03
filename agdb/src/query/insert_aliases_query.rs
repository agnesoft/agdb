use crate::DbError;
use crate::DbErrorType;
use crate::DbImpl;
use crate::QueryIds;
use crate::QueryMut;
use crate::QueryResult;
use crate::SearchQuery;
use crate::StorageData;
use crate::query_builder::search::SearchQueryBuilder;

/// Query to insert or update aliases of existing nodes.
/// All `ids` must exist. None of the `aliases` can be empty.
/// If there is an existing alias for any of the elements it
/// will be overwritten with a new one.
///
/// The result will contain number of aliases inserted/updated but no elements.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[cfg_attr(feature = "derive", derive(agdb::DbSerialize))]
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
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
                    return Err(DbError::query(
                        DbErrorType::NotEnoughData,
                        format!(
                            "Ids ({}) must match aliases ({})",
                            ids.len(),
                            self.aliases.len()
                        ),
                    ));
                }

                for (id, alias) in ids.iter().zip(&self.aliases) {
                    if alias.is_empty() {
                        return Err(DbError::query(
                            DbErrorType::NotAllowed,
                            "Empty alias is not allowed",
                        ));
                    }

                    let db_id = db.db_id(id)?;
                    db.insert_alias(db_id, alias)?;
                    result.result += 1;
                }
            }
            QueryIds::Search(search_query) => {
                let db_ids = search_query.search(db)?;

                if db_ids.len() != self.aliases.len() {
                    return Err(DbError::query(
                        DbErrorType::NotEnoughData,
                        format!(
                            "Search results ({}) must match aliases ({})",
                            db_ids.len(),
                            self.aliases.len()
                        ),
                    ));
                }

                for (db_id, alias) in db_ids.iter().zip(&self.aliases) {
                    if alias.is_empty() {
                        return Err(DbError::query(
                            DbErrorType::NotAllowed,
                            "Empty alias is not allowed",
                        ));
                    }

                    db.insert_alias(*db_id, alias)?;
                    result.result += 1;
                }
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

impl SearchQueryBuilder for InsertAliasesQuery {
    fn search_mut(&mut self) -> &mut SearchQuery {
        if let QueryIds::Search(search) = &mut self.ids {
            search
        } else {
            panic!("Expected search query");
        }
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
    fn search_query_aliases() {
        let test_file = TestFile::new();
        let mut db = Db::new(test_file.file_name()).unwrap();

        db.exec_mut(&crate::InsertNodesQuery {
            count: 2,
            values: crate::QueryValues::Single(vec![]),
            aliases: vec![],
            ids: QueryIds::Ids(vec![]),
        })
        .unwrap();

        let query = InsertAliasesQuery {
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::Elements,
                origin: QueryId::Id(DbId(0)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false,
            }),
            aliases: vec!["alias1".to_string(), "alias2".to_string()],
        };
        let result = query.process(&mut db).unwrap();
        assert_eq!(result.result, 2);
    }

    #[test]
    fn search_query_aliases_length_mismatch() {
        let test_file = TestFile::new();
        let mut db = Db::new(test_file.file_name()).unwrap();

        db.exec_mut(&crate::InsertNodesQuery {
            count: 2,
            values: crate::QueryValues::Single(vec![]),
            aliases: vec![],
            ids: QueryIds::Ids(vec![]),
        })
        .unwrap();

        let query = InsertAliasesQuery {
            ids: QueryIds::Search(SearchQuery {
                algorithm: SearchQueryAlgorithm::Elements,
                origin: QueryId::Id(DbId(0)),
                destination: QueryId::Id(DbId(0)),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
                reverse: false,
            }),
            aliases: vec!["only_one".to_string()],
        };
        assert!(query.process(&mut db).is_err());
    }

    #[test]
    #[should_panic]
    fn missing_search() {
        InsertAliasesQuery {
            ids: QueryIds::Ids(vec![]),
            aliases: vec![],
        }
        .search_mut();
    }
}
