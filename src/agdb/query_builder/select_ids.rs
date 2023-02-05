use crate::query::select_query::SelectQuery;
use crate::query::Query;

pub struct SelectIds(pub SelectQuery);

impl SelectIds {
    pub fn query(self) -> Query {
        Query::Select(self.0)
    }
}
