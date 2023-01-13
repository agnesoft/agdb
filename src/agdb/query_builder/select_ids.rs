use crate::query::select_query::SelectQuery;

pub struct SelectIds(pub SelectQuery);

impl SelectIds {
    pub fn query(self) -> SelectQuery {
        self.0
    }
}
