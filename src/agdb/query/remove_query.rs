use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::QueryMut;
use crate::commands_mut::remove_edge::RemoveEdge;
use crate::commands_mut::remove_node::RemoveNode;
use crate::commands_mut::CommandsMut;
use crate::QueryError;

pub struct RemoveQuery(pub QueryIds);

impl QueryMut for RemoveQuery {
    fn commands(&self) -> Result<Vec<CommandsMut>, QueryError> {
        match &self.0 {
            QueryIds::Ids(ids) => Ok(Self::ids(ids)),
            QueryIds::Search(_) => Err(QueryError::from("Invalid remove query")),
        }
    }
}

impl RemoveQuery {
    fn id(id: &QueryId) -> CommandsMut {
        match id {
            QueryId::Id(db_id) if db_id.0.is_positive() => {
                CommandsMut::RemoveNode(RemoveNode::new(id.clone()))
            }
            QueryId::Id(db_id) => CommandsMut::RemoveEdge(RemoveEdge::new(*db_id)),
            QueryId::Alias(_) => CommandsMut::RemoveNode(RemoveNode::new(id.clone())),
        }
    }

    fn ids(ids: &Vec<QueryId>) -> Vec<CommandsMut> {
        let mut commands = Vec::<CommandsMut>::new();

        for id in ids {
            commands.push(Self::id(id));
        }

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::search_query::SearchQuery;
    use crate::DbId;

    #[test]
    fn valid_id() {
        let query = RemoveQuery(QueryIds::Ids(vec![QueryId::from(1)]));

        assert_eq!(
            query.commands(),
            Ok(vec![CommandsMut::RemoveNode(RemoveNode::new(
                QueryId::from(1)
            ))])
        )
    }

    #[test]
    fn valid_ids() {
        let query = RemoveQuery(QueryIds::Ids(vec![QueryId::from(-3)]));

        assert_eq!(
            query.commands(),
            Ok(vec![CommandsMut::RemoveEdge(RemoveEdge::new(DbId(-3)))])
        )
    }

    #[test]
    fn invalid_query_all() {
        let query = RemoveQuery(QueryIds::Search(SearchQuery {
            origin: QueryId::from(0),
            destination: QueryId::from(0),
            limit: 0,
            offset: 0,
            order_by: vec![],
            conditions: vec![],
        }));

        assert_eq!(
            query.commands().unwrap_err().description,
            QueryError::from("Invalid remove query").description
        );
    }
}
