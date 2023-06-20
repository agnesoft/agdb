use super::query_id::QueryId;
use super::search_query::SearchQuery;
use crate::DbId;

#[derive(Debug, Clone, PartialEq)]
pub enum QueryIds {
    Ids(Vec<QueryId>),
    Search(SearchQuery),
}

impl From<Vec<QueryId>> for QueryIds {
    fn from(value: Vec<QueryId>) -> Self {
        QueryIds::Ids(value)
    }
}

impl From<&[QueryId]> for QueryIds {
    fn from(value: &[QueryId]) -> Self {
        QueryIds::Ids(value.to_vec())
    }
}

impl From<QueryId> for QueryIds {
    fn from(value: QueryId) -> Self {
        QueryIds::Ids(vec![value])
    }
}

impl From<Vec<String>> for QueryIds {
    fn from(value: Vec<String>) -> Self {
        QueryIds::Ids(value.into_iter().map(|v| v.into()).collect())
    }
}

impl From<&[String]> for QueryIds {
    fn from(value: &[String]) -> Self {
        QueryIds::Ids(value.iter().map(|v| v.into()).collect())
    }
}

impl From<Vec<&str>> for QueryIds {
    fn from(value: Vec<&str>) -> Self {
        QueryIds::Ids(value.iter().map(|v| (*v).into()).collect())
    }
}

impl From<Vec<i64>> for QueryIds {
    fn from(value: Vec<i64>) -> Self {
        QueryIds::Ids(value.into_iter().map(|v| v.into()).collect())
    }
}

impl From<Vec<DbId>> for QueryIds {
    fn from(value: Vec<DbId>) -> Self {
        QueryIds::Ids(value.into_iter().map(|v| v.into()).collect())
    }
}

impl From<&[DbId]> for QueryIds {
    fn from(value: &[DbId]) -> Self {
        QueryIds::Ids(value.into_iter().map(|v| v.into()).collect())
    }
}

impl From<i64> for QueryIds {
    fn from(value: i64) -> Self {
        QueryIds::Ids(vec![value.into()])
    }
}

impl From<DbId> for QueryIds {
    fn from(value: DbId) -> Self {
        QueryIds::Ids(vec![value.into()])
    }
}

impl From<&str> for QueryIds {
    fn from(value: &str) -> Self {
        QueryIds::Ids(vec![value.into()])
    }
}

impl From<String> for QueryIds {
    fn from(value: String) -> Self {
        QueryIds::Ids(vec![value.into()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::redundant_clone)]
    fn derived_from_clone() {
        let left = QueryIds::Ids(vec![QueryId::from(0)]);
        let right = left.clone();
        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            QueryIds::Ids(vec![QueryId::from(0)]),
            QueryIds::Ids(vec![QueryId::from(0)])
        );
    }

    #[test]
    fn derived_from_debug() {
        format!("{:?}", QueryIds::Ids(vec![QueryId::from(0)]));
    }
}
