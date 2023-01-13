use crate::query::remove_values_query::RemoveValuesQuery;

pub struct RemoveValuesIds(pub RemoveValuesQuery);

impl RemoveValuesIds {
    pub fn query(self) -> RemoveValuesQuery {
        self.0
    }
}
