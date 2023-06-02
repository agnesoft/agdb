use crate::query::remove_aliases_query::RemoveAliasesQuery;

pub struct RemoveAliases(pub RemoveAliasesQuery);

impl RemoveAliases {
    pub fn query(self) -> RemoveAliasesQuery {
        self.0
    }
}
