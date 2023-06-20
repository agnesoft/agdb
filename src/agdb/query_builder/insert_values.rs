use crate::query::insert_values_query::InsertValuesQuery;
use crate::query::query_ids::QueryIds;
use crate::query::search_query::SearchQuery;

pub struct InsertValuesUniform(pub InsertValuesQuery);

pub struct InsertValuesIds(pub InsertValuesQuery);

pub struct InsertValues(pub InsertValuesQuery);

impl InsertValues {
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> InsertValuesIds {
        self.0.ids = ids.into();

        InsertValuesIds(self.0)
    }

    pub fn search(mut self, query: SearchQuery) -> InsertValuesIds {
        self.0.ids = QueryIds::Search(query);

        InsertValuesIds(self.0)
    }
}

impl InsertValuesUniform {
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> InsertValuesIds {
        self.0.ids = ids.into();

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
