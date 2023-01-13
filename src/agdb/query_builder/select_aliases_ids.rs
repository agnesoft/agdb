use crate::query::select_aliases_query::SelectAliasesQuery;

pub struct SelectAliasesIds(pub SelectAliasesQuery);

impl SelectAliasesIds {
    pub fn query(self) -> SelectAliasesQuery {
        self.0
    }
}
