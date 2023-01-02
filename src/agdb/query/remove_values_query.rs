use super::select_query::SelectQuery;
use super::Query;

pub struct RemoveValuesQuery(pub SelectQuery);

impl Query for RemoveValuesQuery {}
