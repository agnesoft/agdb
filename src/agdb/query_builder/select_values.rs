use super::select_values_from::SelectValuesFrom;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;
use crate::query::select_values_query::SelectValuesQuery;

pub struct SelectValues(pub SelectValuesQuery);

impl SelectValues {
    #[allow(clippy::wrong_self_convention)]
    pub fn from(mut self, id: QueryId) -> SelectValuesFrom {
        self.0.ids = QueryIds::Id(id);

        SelectValuesFrom(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_ids(mut self, ids: &[QueryId]) -> SelectValuesFrom {
        self.0.ids = QueryIds::Ids(ids.to_vec());

        SelectValuesFrom(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_query(mut self, query: SearchQuery) -> SelectValuesFrom {
        self.0.ids = QueryIds::Search(query);

        SelectValuesFrom(self.0)
    }
}
