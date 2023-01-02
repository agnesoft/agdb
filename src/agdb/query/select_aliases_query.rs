use super::Query;

pub struct SelectAliasesQuery {
    pub ids: Vec<u64>,
}

impl Query for SelectAliasesQuery {}
