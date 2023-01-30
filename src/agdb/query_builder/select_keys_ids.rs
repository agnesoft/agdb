use crate::query::{select_keys_query::SelectKeysQuery, Query};

pub struct SelectKeysIds(pub SelectKeysQuery);

impl SelectKeysIds {
    pub fn query(self) -> Query {
        Query::SelectKeys(self.0)
    }
}
