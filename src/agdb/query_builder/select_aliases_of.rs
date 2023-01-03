use crate::query::select_aliases_query::SelectAliasesQuery;

pub struct SelectAliasesOf(pub SelectAliasesQuery);

impl SelectAliasesOf {
    pub fn query(self) -> SelectAliasesQuery {
        self.0
    }
}
