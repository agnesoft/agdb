use crate::query::select_aliases_query::SelectAliasesQuery;
use crate::query::Query;

pub struct SelectAliasesIds(pub SelectAliasesQuery);

impl SelectAliasesIds {
    pub fn query(self) -> Query {
        Query::SelectAliases(self.0)
    }
}
