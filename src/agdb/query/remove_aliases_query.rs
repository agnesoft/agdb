use super::QueryMut;
use crate::commands_mut::remove_alias::RemoveAlias;
use crate::commands_mut::CommandsMut;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

pub struct RemoveAliasesQuery {
    pub aliases: Vec<String>,
}

impl QueryMut for RemoveAliasesQuery {
    fn commands(&self) -> Result<Vec<CommandsMut>, QueryError> {
        Ok(self
            .aliases
            .iter()
            .map(|alias| CommandsMut::RemoveAlias(RemoveAlias::new(alias.clone())))
            .collect())
    }

    fn process(&self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        for alias in &self.aliases {
            if db.remove_alias(alias)? {
                result.result -= 1;
            }
        }

        Ok(())
    }
}
