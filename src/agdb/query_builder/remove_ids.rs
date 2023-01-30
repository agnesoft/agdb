use crate::query::remove_query::RemoveQuery;
use crate::query::Query;

pub struct RemoveIds(pub RemoveQuery);

impl RemoveIds {
    pub fn query(self) -> Query {
        Query::Remove(self.0)
    }
}
