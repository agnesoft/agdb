use crate::query::insert_aliases_query::InsertAliasesQuery;

pub struct InsertAliasesOf(pub InsertAliasesQuery);

impl InsertAliasesOf {
    pub fn query(self) -> InsertAliasesQuery {
        self.0
    }
}
