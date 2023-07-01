use super::query_id::QueryId;
use super::query_ids::QueryIds;
use crate::Db;
use crate::DbElement;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;

pub struct SelectAliasesQuery(pub QueryIds);

impl Query for SelectAliasesQuery {
    fn process(&self, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        match &self.0 {
            QueryIds::Ids(ids) => {
                result.elements.reserve(ids.len());
                result.result += ids.len() as i64;

                for id in ids {
                    match id {
                        QueryId::Id(db_id) => result.elements.push(DbElement {
                            id: *db_id,
                            values: vec![("alias", db.alias(*db_id)?).into()],
                        }),
                        QueryId::Alias(alias) => result.elements.push(DbElement {
                            id: db.db_id(id)?,
                            values: vec![("alias", alias).into()],
                        }),
                    }
                }
            }
            QueryIds::Search(search_query) => {
                for db_id in search_query.search(db)? {
                    if let Ok(alias) = db.alias(db_id) {
                        result.elements.push(DbElement {
                            id: db_id,
                            values: vec![("alias", alias).into()],
                        });
                    }
                }

                result.result = result.elements.len() as i64;
            }
        }

        Ok(())
    }
}
