use super::query_ids::QueryIds;
use super::Query;

pub struct RemoveQuery(pub QueryIds);

impl Query for RemoveQuery {}
