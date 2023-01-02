use super::remove_alias::RemoveAlias;
use crate::query::remove_aliases_query::RemoveAliasesQuery;

pub struct Remove {}

impl Remove {
    pub fn alias(self, name: &str) -> RemoveAlias {
        RemoveAlias(RemoveAliasesQuery {
            aliases: vec![name.to_string()],
        })
    }

    pub fn aliases(self, names: &[String]) -> RemoveAlias {
        RemoveAlias(RemoveAliasesQuery {
            aliases: names.to_vec(),
        })
    }
}
