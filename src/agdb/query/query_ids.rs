use super::query_id::QueryId;
use super::search_query::SearchQuery;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryIds {
    Ids(Vec<QueryId>),
    Search(SearchQuery),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", QueryIds::Ids(vec![QueryId::from(0)]));
    }
}
