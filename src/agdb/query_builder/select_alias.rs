use super::select_aliases_of::SelectAliasesOf;
use crate::query::query_ids::QueryIds;
use crate::query::select_aliases_query::SelectAliasesQuery;

pub struct SelectAlias(pub SelectAliasesQuery);

impl SelectAlias {
    pub fn of(mut self, id: u64) -> SelectAliasesOf {
        self.0.ids = QueryIds::Id(id.into());

        SelectAliasesOf(self.0)
    }
}
