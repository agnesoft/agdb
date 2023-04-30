use crate::db::db_context::Context;
use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertAlias {
    id: Option<DbId>,
    alias: String,
    result: bool,
}

impl InsertAlias {
    pub(crate) fn new(alias: String, id: Option<DbId>) -> Self {
        Self {
            id,
            alias,
            result: id.is_some(),
        }
    }

    pub(crate) fn redo(
        &mut self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &Context,
    ) -> Result<(), QueryError> {
        if self.alias.is_empty() {
            return Err(QueryError::from("Empty alias is not allowed"));
        }

        let id = if let Some(id) = self.id {
            id
        } else {
            self.id = Some(context.id);
            context.id
        };

        db.aliases.insert(&self.alias, &id)?;

        if self.result {
            result.result += 1;
        }

        Ok(())
    }

    pub(crate) fn undo(&mut self, db: &mut Db) -> Result<(), QueryError> {
        Ok(db.aliases.remove_key(&self.alias)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", InsertAlias::new(String::new(), None));
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertAlias::new(String::new(), None),
            InsertAlias::new(String::new(), None)
        );
    }
}
