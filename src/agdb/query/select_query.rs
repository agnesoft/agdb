use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::Query;
use crate::commands::select_id::SelectId;
use crate::commands::Commands;
use crate::QueryError;

pub struct SelectQuery(pub QueryIds);

impl Query for SelectQuery {
    fn commands(&self) -> Result<Vec<Commands>, QueryError> {
        match &self.0 {
            QueryIds::Ids(ids) => Ok(Self::ids(ids)),
            QueryIds::Search(_) => Err(QueryError::from("Invalid select query")),
        }
    }
}

impl SelectQuery {
    fn ids(ids: &[QueryId]) -> Vec<Commands> {
        ids.iter()
            .map(|id| Commands::SelectId(SelectId { id: id.clone() }))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::search_query::SearchQuery;

    #[test]
    fn valid_id() {
        let query = SelectQuery(QueryIds::Ids(vec![QueryId::from(0)]));

        assert_eq!(
            query.commands(),
            Ok(vec![Commands::SelectId(SelectId {
                id: QueryId::from(0)
            })])
        )
    }

    #[test]
    fn valid_ids() {
        let query = SelectQuery(QueryIds::Ids(vec![QueryId::from(0)]));

        assert_eq!(
            query.commands(),
            Ok(vec![Commands::SelectId(SelectId {
                id: QueryId::from(0)
            })])
        )
    }

    #[test]
    fn invalid_query_all() {
        let query = SelectQuery(QueryIds::Search(SearchQuery {
            origin: QueryId::from(0),
            destination: QueryId::from(0),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        }));

        assert_eq!(
            query.commands().unwrap_err().description,
            QueryError::from("Invalid select query").description
        );
    }
}
