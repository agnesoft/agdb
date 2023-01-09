use crate::query::select_key_count_query::SelectKeyCountQuery;

pub struct SelectKeyCountFrom(pub SelectKeyCountQuery);

impl SelectKeyCountFrom {
    pub fn query(self) -> SelectKeyCountQuery {
        self.0
    }
}
