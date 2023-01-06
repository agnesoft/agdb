use super::query_ids::QueryIds;
use super::Query;

pub struct SelectKeysQuery(pub QueryIds);

impl Query for SelectKeysQuery {}
