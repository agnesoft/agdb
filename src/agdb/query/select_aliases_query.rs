use super::query_id::QueryId;
use super::query_ids::QueryIds;
use crate::Db;
use crate::DbElement;
use crate::Query;
use crate::QueryError;
use crate::QueryResult;

pub struct SelectAliasesQuery {
    pub ids: QueryIds,
}

impl Query for SelectAliasesQuery {
    fn process(&self, db: &Db, result: &mut QueryResult) -> Result<(), QueryError> {
        match &self.ids {
            QueryIds::Ids(ids) => {
                for id in ids {
                    match id {
                        QueryId::Id(db_id) => result.elements.push(DbElement {
                            index: *db_id,
                            values: vec![("alias", db.alias(*db_id)?).into()],
                        }),
                        QueryId::Alias(alias) => result.elements.push(DbElement {
                            index: db.db_id(id)?,
                            values: vec![("alias", alias).into()],
                        }),
                    }
                    result.result += 1;
                }
                Ok(())
            }
            QueryIds::Search(_) => Err(QueryError::from("Invalid select aliases query")),
        }
    }
}
