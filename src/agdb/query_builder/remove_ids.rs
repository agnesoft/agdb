use crate::query::remove_query::RemoveQuery;

pub struct RemoveIds(pub RemoveQuery);

impl RemoveIds {
    pub fn query(self) -> RemoveQuery {
        self.0
    }
}
