use crate::query::insert_aliases_query::InsertAliasesQuery;
use crate::query::Query;

pub struct InsertAliasesIds(pub InsertAliasesQuery);

impl InsertAliasesIds {
    pub fn query(self) -> Query {
        Query::InsertAliases(self.0)
    }
}
