use crate::query::insert_aliases_query::InsertAliasesQuery;

pub struct InsertAliasesIds(pub InsertAliasesQuery);

impl InsertAliasesIds {
    pub fn query(self) -> InsertAliasesQuery {
        self.0
    }
}
