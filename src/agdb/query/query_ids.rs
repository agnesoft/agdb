use super::query_id::QueryId;
use super::search_query::SearchQuery;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryIds {
    Id(QueryId),
    Ids(Vec<QueryId>),
    Search(SearchQuery),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", QueryIds::Id(QueryId::Id(0)));
    }

    #[test]
    fn derived_from_clone() {
        let left = QueryIds::Id(QueryId::Id(0));
        let right = left.clone();

        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(QueryIds::Id(QueryId::Id(0)), QueryIds::Id(QueryId::Id(0)));
    }
}
