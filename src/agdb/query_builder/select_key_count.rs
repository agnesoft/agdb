use crate::query::query_ids::QueryIds;
use crate::query::select_key_count_query::SelectKeyCountQuery;

pub struct SelectKeyCount(pub SelectKeyCountQuery);

pub struct SelectKeyCountIds(pub SelectKeyCountQuery);

impl SelectKeyCount {
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectKeyCountIds {
        self.0 .0 = ids.into();

        SelectKeyCountIds(self.0)
    }
}

impl SelectKeyCountIds {
    pub fn query(self) -> SelectKeyCountQuery {
        self.0
    }
}
