use crate::query::remove_values_query::RemoveValuesQuery;
use crate::query::Query;

pub struct RemoveValuesIds(pub RemoveValuesQuery);

impl RemoveValuesIds {
    pub fn query(self) -> Query {
        Query::RemoveValues(self.0)
    }
}
