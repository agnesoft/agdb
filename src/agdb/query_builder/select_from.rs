use crate::query::select_query::SelectQuery;

pub struct SelectFrom(pub SelectQuery);

impl SelectFrom {
    pub fn query(self) -> SelectQuery {
        self.0
    }
}
