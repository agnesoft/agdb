use super::search_query::SearchQuery;
use super::Query;

pub struct SelectCountQuery(pub SearchQuery);

impl Query for SelectCountQuery {}
