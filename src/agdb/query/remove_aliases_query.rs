use crate::commands::remove_alias::RemoveAlias;
use crate::commands::Commands;

pub struct RemoveAliasesQuery {
    pub aliases: Vec<String>,
}

impl RemoveAliasesQuery {
    pub fn commands(&self) -> Vec<Commands> {
        self.aliases
            .iter()
            .map(|alias| {
                Commands::RemoveAlias(RemoveAlias {
                    alias: alias.clone(),
                })
            })
            .collect()
    }
}
