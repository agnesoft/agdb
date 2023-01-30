use crate::query::select_values_query::SelectValuesQuery;
use crate::query::Query;

pub struct SelectValuesIds(pub SelectValuesQuery);

impl SelectValuesIds {
    pub fn query(self) -> Query {
        Query::SelectValues(self.0)
    }
}
