use crate::query::insert_values_query::InsertValuesQuery;
use crate::query::Query;

pub struct InsertValuesIds(pub InsertValuesQuery);

impl InsertValuesIds {
    pub fn query(self) -> Query {
        Query::InsertValues(self.0)
    }
}
