use crate::query::insert_values_query::InsertValuesQuery;

pub struct InsertValuesIds(pub InsertValuesQuery);

impl InsertValuesIds {
    pub fn query(self) -> InsertValuesQuery {
        self.0
    }
}
