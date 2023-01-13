use super::remove_values_from::RemoveValuesFrom;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::remove_values_query::RemoveValuesQuery;
use crate::query::search_query::SearchQuery;

pub struct RemoveValues(pub RemoveValuesQuery);

impl RemoveValues {
    pub fn from(mut self, id: QueryId) -> RemoveValuesFrom {
        self.0 .0.ids = QueryIds::Id(id);

        RemoveValuesFrom(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_ids(mut self, ids: &[QueryId]) -> RemoveValuesFrom {
        self.0 .0.ids = QueryIds::Ids(ids.to_vec());

        RemoveValuesFrom(self.0)
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_search(mut self, query: SearchQuery) -> RemoveValuesFrom {
        self.0 .0.ids = QueryIds::Search(query);

        RemoveValuesFrom(self.0)
    }
}
