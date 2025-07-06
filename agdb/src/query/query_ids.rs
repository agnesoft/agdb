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
#[cfg_attr(feature = "derive", derive(agdb::AgdbDeSerialize))]
#[cfg_attr(feature = "api", derive(agdb::ApiDef))]
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

impl<T: Into<QueryId>> From<Vec<T>> for QueryIds {
    fn from(value: Vec<T>) -> Self {
        QueryIds::Ids(value.into_iter().map(|v| v.into()).collect())
    }
}

impl<T: Into<QueryId> + Clone> From<&Vec<T>> for QueryIds {
    fn from(value: &Vec<T>) -> Self {
        QueryIds::Ids(value.iter().map(|v| v.clone().into()).collect())
    }
}

impl<T: Into<QueryId> + Clone> From<&[T]> for QueryIds {
    fn from(value: &[T]) -> Self {
        QueryIds::Ids(value.iter().map(|v| v.clone().into()).collect())
    }
}

impl<T: Into<QueryId> + Clone, const N: usize> From<[T; N]> for QueryIds {
    fn from(value: [T; N]) -> Self {
        QueryIds::Ids(value.into_iter().map(|v| v.into()).collect())
    }
}

impl<T: Into<QueryId>> From<T> for QueryIds {
    fn from(value: T) -> Self {
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
    use crate::DbId;
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
        let _ = format!("{:?}", QueryIds::Ids(vec![QueryId::from(0)]));
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

    #[test]
    fn into_ids() {
        let _ids = QueryIds::from(vec![QueryId::from(0)]);
        let _ids = QueryIds::from(vec![0]);
        let _ids = QueryIds::from(vec!["alias"]);
        let _ids = QueryIds::from(vec![DbId(0)]);
        let _ids = QueryIds::from(&vec![QueryId::from(0)]);
        let _ids = QueryIds::from(&vec![0]);
        let _ids = QueryIds::from(&vec!["alias"]);
        let _ids = QueryIds::from(&vec![DbId(0)]);
        let _ids = QueryIds::from([QueryId::from(0)].as_slice());
        let _ids = QueryIds::from([0].as_slice());
        let _ids = QueryIds::from(["alias"].as_slice());
        let _ids = QueryIds::from([DbId(0)].as_slice());
        let _ids = QueryIds::from([QueryId::from(0)]);
        let _ids = QueryIds::from([0]);
        let _ids = QueryIds::from(["alias"]);
        let _ids = QueryIds::from([DbId(0)]);
        let _ids = QueryIds::from(0);
        let _ids = QueryIds::from("alias");
        let _ids = QueryIds::from("alias".to_string());
        let _ids = QueryIds::from(&"alias".to_string());
        let _ids = QueryIds::from(DbId(0));
    }
}
