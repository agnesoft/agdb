use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::remove_values_query::RemoveValuesQuery;
use crate::query::search_query::SearchQuery;

pub struct RemoveValues(pub RemoveValuesQuery);

pub struct RemoveValuesIds(pub RemoveValuesQuery);

impl RemoveValues {
    pub fn ids(mut self, ids: &[QueryId]) -> RemoveValuesIds {
        self.0 .0.ids = QueryIds::Ids(ids.to_vec());

        RemoveValuesIds(self.0)
    }

    pub fn search(mut self, query: SearchQuery) -> RemoveValuesIds {
        self.0 .0.ids = QueryIds::Search(query);

        RemoveValuesIds(self.0)
    }
}

impl RemoveValuesIds {
    pub fn query(self) -> RemoveValuesQuery {
        self.0
    }
}
