use super::query_ids::QueryIds;
use crate::Db;
use crate::DbElement;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;

pub struct SelectKeyCountQuery(pub QueryIds);

impl Query for SelectKeyCountQuery {
    fn process(&self, db: &Db) -> Result<QueryResult, QueryError> {
        let mut result = QueryResult::default();

        let db_ids = match &self.0 {
            QueryIds::Ids(ids) => {
                let mut db_ids = vec![];
                db_ids.reserve(ids.len());

                for query_id in ids {
                    db_ids.push(db.db_id(query_id)?);
                }

                db_ids
            }
            QueryIds::Search(search_query) => search_query.search(db)?,
        };

        result.elements.reserve(db_ids.len());
        result.result = db_ids.len() as i64;

        for db_id in db_ids {
            result.elements.push(DbElement {
                id: db_id,
                values: vec![("key_count", db.key_count(db_id)?).into()],
            });
        }

        Ok(result)
    }
}
