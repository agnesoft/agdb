use crate::DbId;
use crate::QueryId;
use crate::QueryResult;
use crate::SearchQuery;

/// List of database ids used in queries. It
/// can either represent a list of [`QueryId`]s
/// or a search query. Search query allows query
/// nesting and sourcing the ids dynamically for
/// another query most commonly with the
/// select queries.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub enum QueryIds {
    /// List of [`QueryId`]s
    Ids(Vec<QueryId>),

    /// Search query
    Search(SearchQuery),
}

impl QueryIds {
    pub(crate) fn get_ids(self) -> Vec<QueryId> {
        match self {
            QueryIds::Ids(ids) => ids,
            QueryIds::Search(_) => vec![],
        }
    }
}

impl From<Vec<QueryId>> for QueryIds {
    fn from(value: Vec<QueryId>) -> Self {
        QueryIds::Ids(value)
    }
}

impl From<Vec<String>> for QueryIds {
    fn from(value: Vec<String>) -> Self {
        QueryIds::Ids(value.into_iter().map(|v| v.into()).collect())
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

impl From<QueryResult> for QueryIds {
    fn from(value: QueryResult) -> Self {
        Self::from(&value)
    }
}

impl From<&QueryResult> for QueryIds {
    fn from(value: &QueryResult) -> Self {
        QueryIds::Ids(value.elements.iter().map(|e| QueryId::from(e.id)).collect())
    }
}

impl From<SearchQuery> for QueryIds {
    fn from(query: SearchQuery) -> Self {
        QueryIds::Search(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::search_query::SearchQueryAlgorithm;

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

    #[test]
    fn get_ids_from_search() {
        let ids = QueryIds::Search(SearchQuery {
            algorithm: SearchQueryAlgorithm::BreadthFirst,
            origin: QueryId::Id(DbId(0)),
            destination: QueryId::Id(DbId(0)),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        })
        .get_ids();

        assert_eq!(ids, vec![]);
    }
}
