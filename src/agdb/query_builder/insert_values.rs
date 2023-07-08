use crate::query::insert_values_query::InsertValuesQuery;
use crate::query::query_ids::QueryIds;

pub struct InsertValues(pub InsertValuesQuery);

pub struct InsertValuesIds(pub InsertValuesQuery);

impl InsertValues {
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> InsertValuesIds {
        self.0.ids = ids.into();

        InsertValuesIds(self.0)
    }
}

impl InsertValuesIds {
    pub fn query(self) -> InsertValuesQuery {
        self.0
    }
}
