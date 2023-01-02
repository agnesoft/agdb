use super::Query;

pub struct RemoveAliasesQuery {
    pub aliases: Vec<String>,
}

impl Query for RemoveAliasesQuery {}
