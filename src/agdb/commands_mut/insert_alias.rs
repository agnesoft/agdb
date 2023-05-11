use crate::query::query_id::QueryId;
use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertAlias {
    id: QueryId,
    db_id: DbId,
    alias: String,
    old_alias: String,
}

impl InsertAlias {
    pub(crate) fn new(id: QueryId, alias: String) -> Self {
        Self {
            id,
            db_id: DbId(0),
            alias,
            old_alias: String::new(),
        }
    }

    pub(crate) fn redo(&mut self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        if self.alias.is_empty() {
            return Err(QueryError::from("Empty alias is not allowed"));
        }

        self.db_id = match &self.id {
            QueryId::Id(id) => {
                if let Some(old_alias) = db.aliases.key(id)? {
                    self.old_alias = old_alias;
                } else {
                    let _ = db
                        .indexes
                        .value(id)?
                        .ok_or(QueryError::from(format!("Id '{}' not found", id.0)))?;
                }

                *id
            }
            QueryId::Alias(old_alias) => {
                self.old_alias = old_alias.clone();
                db.aliases
                    .value(&self.old_alias)?
                    .ok_or(QueryError::from(format!("Alias '{}' not found", old_alias)))?
            }
        };

        db.aliases.insert(&self.alias, &self.db_id)?;
        result.result += 1;

        Ok(())
    }

    pub(crate) fn undo(self, db: &mut Db) -> Result<(), QueryError> {
        db.aliases.remove_key(&self.alias)?;

        if !self.old_alias.is_empty() {
            db.aliases.insert(&self.old_alias, &self.db_id)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            InsertAlias::new(QueryId::Id(DbId(0)), String::new())
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertAlias::new(QueryId::Id(DbId(0)), String::new()),
            InsertAlias::new(QueryId::Id(DbId(0)), String::new())
        );
    }
}
