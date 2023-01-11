use super::select_values_query::SelectValuesQuery;
use super::Query;

pub struct RemoveValuesQuery(pub SelectValuesQuery);

impl Query for RemoveValuesQuery {}
