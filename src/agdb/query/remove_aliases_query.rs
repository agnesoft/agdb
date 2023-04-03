use super::QueryMut;
use crate::commands_mut::remove_alias::RemoveAlias;
use crate::commands_mut::remove_alias_result::RemoveAliasResult;
use crate::commands_mut::CommandsMut;
use crate::QueryError;

pub struct RemoveAliasesQuery {
    pub aliases: Vec<String>,
}

impl QueryMut for RemoveAliasesQuery {
    fn commands(&self) -> Result<Vec<CommandsMut>, QueryError> {
        Ok(self
            .aliases
            .iter()
            .map(|alias| {
                CommandsMut::RemoveAliasResult(RemoveAliasResult(RemoveAlias {
                    alias: alias.clone(),
                }))
            })
            .collect())
    }
}
