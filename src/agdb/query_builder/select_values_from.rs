use crate::query::select_values_query::SelectValuesQuery;

pub struct SelectValuesFrom(pub SelectValuesQuery);

impl SelectValuesFrom {
    pub fn query(self) -> SelectValuesQuery {
        self.0
    }
}
