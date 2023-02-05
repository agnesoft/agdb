use crate::query::remove_aliases_query::RemoveAliasesQuery;
use crate::query::Query;

pub struct RemoveAlias(pub RemoveAliasesQuery);

impl RemoveAlias {
    pub fn query(self) -> Query {
        Query::RemoveAliases(self.0)
    }
}
