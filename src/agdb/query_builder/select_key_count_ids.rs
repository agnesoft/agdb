use crate::query::select_key_count_query::SelectKeyCountQuery;

pub struct SelectKeyCountIds(pub SelectKeyCountQuery);

impl SelectKeyCountIds {
    pub fn query(self) -> SelectKeyCountQuery {
        self.0
    }
}
