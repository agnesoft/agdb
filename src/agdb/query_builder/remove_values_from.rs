use crate::query::remove_values_query::RemoveValuesQuery;

pub struct RemoveValuesFrom(pub RemoveValuesQuery);

impl RemoveValuesFrom {
    pub fn query(self) -> RemoveValuesQuery {
        self.0
    }
}
