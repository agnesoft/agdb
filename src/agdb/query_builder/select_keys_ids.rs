use crate::query::select_keys_query::SelectKeysQuery;

pub struct SelectKeysIds(pub SelectKeysQuery);

impl SelectKeysIds {
    pub fn query(self) -> SelectKeysQuery {
        self.0
    }
}
