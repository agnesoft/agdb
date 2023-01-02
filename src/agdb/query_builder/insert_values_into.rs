use crate::query::insert_values_query::InsertValuesQuery;

pub struct InsertValuesInto(pub InsertValuesQuery);

impl InsertValuesInto {
    pub fn query(self) -> InsertValuesQuery {
        self.0
    }
}
