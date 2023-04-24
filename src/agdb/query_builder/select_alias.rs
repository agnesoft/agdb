use super::select_aliases_ids::SelectAliasesIds;
use crate::query::query_ids::QueryIds;
use crate::query::select_aliases_query::SelectAliasesQuery;

pub struct SelectAlias(pub SelectAliasesQuery);

impl SelectAlias {
    pub fn id(mut self, id: i64) -> SelectAliasesIds {
        self.0.ids = QueryIds::Ids(vec![id.into()]);

        SelectAliasesIds(self.0)
    }
}
