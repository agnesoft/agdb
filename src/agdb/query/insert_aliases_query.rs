use super::query_ids::QueryIds;
use super::QueryMut;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

pub struct InsertAliasesQuery {
    pub ids: QueryIds,
    pub aliases: Vec<String>,
}

impl QueryMut for InsertAliasesQuery {
    fn process(&self, db: &mut Db) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        match &self.ids {
            QueryIds::Ids(ids) => {
                if ids.len() != self.aliases.len() {
                    return Err(QueryError::from(
                        "Ids and aliases must have the same length",
                    ));
                }

                for (id, alias) in ids.iter().zip(&self.aliases) {
                    if alias.is_empty() {
                        return Err(QueryError::from("Empty alias is not allowed"));
                    }

                    let db_id = db.db_id(id)?;
                    db.insert_alias(db_id, alias)?;
                    result.result += 1;
                }
            }
            QueryIds::Search(_) => {
                return Err(QueryError::from(
                    "Insert aliases query does not support search queries",
                ));
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::query_id::QueryId;
    use crate::query::search_query::SearchQuery;
    use crate::test_utilities::test_file::TestFile;
    use crate::DbId;
    use crate::SearchQueryAlgorithm;

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
            QueryError::from("Insert aliases query does not support search queries")
        );
    }
}
