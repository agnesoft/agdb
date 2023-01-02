use super::query_ids::QueryIds;
use super::Query;

pub struct SelectKeyCountQuery(pub QueryIds);

impl Query for SelectKeyCountQuery {}
