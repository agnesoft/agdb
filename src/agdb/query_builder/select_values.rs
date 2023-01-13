use super::select_values_ids::SelectValuesIds;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::query::select_values_query::SelectValuesQuery;

pub struct SelectValues(pub SelectValuesQuery);

impl SelectValues {
    pub fn id(mut self, id: QueryId) -> SelectValuesIds {
        self.0.ids = QueryIds::Id(id);

        SelectValuesIds(self.0)
    }

    pub fn ids(mut self, ids: &[QueryId]) -> SelectValuesIds {
        self.0.ids = QueryIds::Ids(ids.to_vec());

        SelectValuesIds(self.0)
    }

    pub fn search(mut self, query: SearchQuery) -> SelectValuesIds {
        self.0.ids = QueryIds::Search(query);

        SelectValuesIds(self.0)
    }
}
