use crate::query::select_keys_query::SelectKeysQuery;

pub struct SelectKeysFrom(pub SelectKeysQuery);

impl SelectKeysFrom {
    pub fn query(self) -> SelectKeysQuery {
        self.0
    }
}
