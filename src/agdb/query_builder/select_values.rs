use crate::query::query_ids::QueryIds;
use crate::query::select_values_query::SelectValuesQuery;

pub struct SelectValues(pub SelectValuesQuery);

pub struct SelectValuesIds(pub SelectValuesQuery);

impl SelectValues {
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectValuesIds {
        self.0.ids = ids.into();

        SelectValuesIds(self.0)
    }
}

impl SelectValuesIds {
    pub fn query(self) -> SelectValuesQuery {
        self.0
    }
}
