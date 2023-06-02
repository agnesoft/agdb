use crate::query::insert_values_query::InsertValuesQuery;
use crate::query::query_id::QueryId;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;

pub struct InsertValuesUniform(pub InsertValuesQuery);

pub struct InsertValuesIds(pub InsertValuesQuery);

pub struct InsertValues(pub InsertValuesQuery);

impl InsertValues {
    pub fn ids(mut self, ids: &[QueryId]) -> InsertValuesIds {
        self.0.ids = QueryIds::Ids(ids.to_vec());

        InsertValuesIds(self.0)
    }

    pub fn search(mut self, query: SearchQuery) -> InsertValuesIds {
        self.0.ids = QueryIds::Search(query);

        InsertValuesIds(self.0)
    }
}

impl InsertValuesUniform {
    pub fn ids(mut self, ids: &[QueryId]) -> InsertValuesIds {
        self.0.ids = QueryIds::Ids(ids.to_vec());

        InsertValuesIds(self.0)
    }

    pub fn search(mut self, query: SearchQuery) -> InsertValuesIds {
        self.0.ids = QueryIds::Search(query);

        InsertValuesIds(self.0)
    }
}

impl InsertValuesIds {
    pub fn query(self) -> InsertValuesQuery {
        self.0
    }
}
