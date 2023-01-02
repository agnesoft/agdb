use crate::query::insert_aliases_query::InsertAliasQuery;

pub struct InsertAliasOf(pub InsertAliasQuery);

impl InsertAliasOf {
    pub fn query(self) -> InsertAliasQuery {
        self.0
    }
}
