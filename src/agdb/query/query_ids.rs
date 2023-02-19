use super::query_id::QueryId;
use super::search_query::SearchQuery;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryIds {
    All,
    Id(QueryId),
    Ids(Vec<QueryId>),
    Search(SearchQuery),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", QueryIds::All);
    }

    #[test]
    fn derived_from_clone() {
        let left = QueryIds::All;
        let right = left.clone();

        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(QueryIds::All, QueryIds::All);
    }
}
