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
    fn process(&self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
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

                Ok(())
            }
            QueryIds::Search(_) => Err(QueryError::from("Invalid insert aliases query")),
        }
    }
}
