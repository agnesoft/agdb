use crate::query::select_key_count_query::SelectKeyCountQuery;
use crate::query::Query;

pub struct SelectKeyCountIds(pub SelectKeyCountQuery);

impl SelectKeyCountIds {
    pub fn query(self) -> Query {
        Query::SelectKeyCount(self.0)
    }
}
