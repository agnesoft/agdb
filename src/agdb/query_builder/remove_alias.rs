use crate::query::remove_aliases_query::RemoveAliasesQuery;

pub struct RemoveAlias(pub RemoveAliasesQuery);

impl RemoveAlias {
    pub fn query(self) -> RemoveAliasesQuery {
        self.0
    }
}
