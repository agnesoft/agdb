use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveAlias {
    id: DbId,
    alias: String,
}

impl RemoveAlias {
    pub(crate) fn new(alias: String) -> Self {
        Self { id: DbId(0), alias }
    }

    pub(crate) fn redo(&mut self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        if let Some(id) = db.aliases.value(&self.alias)? {
            self.id = id;
            db.aliases.remove_key(&self.alias)?;
            result.result -= 1;
        }

        Ok(())
    }

    pub(crate) fn undo(self, db: &mut Db) -> Result<(), QueryError> {
        if self.id.0 != 0 && !self.alias.is_empty() {
            db.aliases.insert(&self.alias, &self.id)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveAlias::new(String::new()));
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveAlias::new(String::new()),
            RemoveAlias::new(String::new())
        );
    }
}
