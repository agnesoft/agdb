use crate::query::query_ids::QueryIds;
use crate::query::remove_values_query::RemoveValuesQuery;

pub struct RemoveValues(pub RemoveValuesQuery);

pub struct RemoveValuesIds(pub RemoveValuesQuery);

impl RemoveValues {
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> RemoveValuesIds {
        self.0 .0.ids = ids.into();

        RemoveValuesIds(self.0)
    }
}

impl RemoveValuesIds {
    pub fn query(self) -> RemoveValuesQuery {
        self.0
    }
}
