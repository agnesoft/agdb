use crate::query::select_values_query::SelectValuesQuery;

pub struct SelectValuesIds(pub SelectValuesQuery);

impl SelectValuesIds {
    pub fn query(self) -> SelectValuesQuery {
        self.0
    }
}
