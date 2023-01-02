use super::{query_ids::QueryIds, Query};

pub struct SelectKeysQuery(pub QueryIds);

impl Query for SelectKeysQuery {}
