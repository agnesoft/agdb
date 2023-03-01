use crate::commands::remove_alias::RemoveAlias;
use crate::commands::Commands;
use crate::QueryError;

pub struct RemoveAliasesQuery {
    pub aliases: Vec<String>,
}

impl RemoveAliasesQuery {
    pub fn commands(&self) -> Result<Vec<Commands>, QueryError> {
        Ok(self
            .aliases
            .iter()
            .map(|alias| {
                Commands::RemoveAlias(RemoveAlias {
                    alias: alias.clone(),
                })
            })
            .collect())
    }
}
