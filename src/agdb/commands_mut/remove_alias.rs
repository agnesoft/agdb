use crate::db::db_context::Context;
use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveAlias {
    id: Option<DbId>,
    alias: String,
    result: bool,
}

impl RemoveAlias {
    pub(crate) fn new(alias: String, id: Option<DbId>, result: bool) -> Self {
        Self { id, alias, result }
    }

    pub(crate) fn redo(
        &mut self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &mut Context,
    ) -> Result<(), QueryError> {
        if let Some(id) = &self.id {
            if let Some(alias) = db.aliases.key(id)? {
                self.alias = alias;
                db.aliases.remove_value(id)?;
            }
        } else if let Some(id) = db.aliases.value(&self.alias)? {
            self.id = Some(id);
            context.id = id;
            db.aliases.remove_key(&self.alias)?;

            if self.result {
                result.result -= 1;
            }
        }

        Ok(())
    }

    pub(crate) fn undo(&mut self, db: &mut Db) -> Result<(), QueryError> {
        if let Some(id) = &self.id {
            if !self.alias.is_empty() {
                db.aliases.insert(&self.alias, id)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveAlias::new(String::new(), None, false));
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveAlias::new(String::new(), None, false),
            RemoveAlias::new(String::new(), None, false)
        );
    }
}
