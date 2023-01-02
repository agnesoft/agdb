use super::insert_values_into::InsertValuesInto;
use crate::query::insert_values_query::InsertValuesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;

pub struct InsertValuesMulti(pub InsertValuesQuery);

impl InsertValuesMulti {
    pub fn into_ids(mut self, ids: &[QueryId]) -> InsertValuesInto {
        self.0.ids = QueryIds::Ids(ids.to_vec());

        InsertValuesInto(self.0)
    }

    pub fn into_query(mut self, query: SearchQuery) -> InsertValuesInto {
        self.0.ids = QueryIds::Search(query);

        InsertValuesInto(self.0)
    }
}
