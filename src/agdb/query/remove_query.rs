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
            QueryIds::Id(id) => Ok(Self::id(id)),
            QueryIds::Ids(ids) => Ok(Self::ids(ids)),
            QueryIds::All | QueryIds::Search(_) => Err(QueryError::from("Invalid remove query")),
        }
    }
}

impl RemoveQuery {
    fn id(id: &QueryId) -> Vec<CommandsMut> {
        if id.is_node() {
            vec![CommandsMut::RemoveNode(RemoveNode { id: id.clone() })]
        } else {
            vec![CommandsMut::RemoveEdge(RemoveEdge { id: id.clone() })]
        }
    }

    fn ids(ids: &Vec<QueryId>) -> Vec<CommandsMut> {
        let mut commands = Vec::<CommandsMut>::new();

        for id in ids {
            if id.is_node() {
                commands.push(CommandsMut::RemoveNode(RemoveNode { id: id.clone() }));
            } else {
                commands.push(CommandsMut::RemoveEdge(RemoveEdge { id: id.clone() }));
            }
        }

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_id() {
        let query = RemoveQuery(QueryIds::Id(QueryId::Id(1)));

        assert_eq!(
            query.commands(),
            Ok(vec![CommandsMut::RemoveNode(RemoveNode {
                id: QueryId::Id(1)
            })])
        )
    }

    #[test]
    fn valid_ids() {
        let query = RemoveQuery(QueryIds::Ids(vec![QueryId::Id(1)]));

        assert_eq!(
            query.commands(),
            Ok(vec![CommandsMut::RemoveNode(RemoveNode {
                id: QueryId::Id(1)
            })])
        )
    }

    #[test]
    fn invalid_query_all() {
        let query = RemoveQuery(QueryIds::All);

        assert_eq!(
            query.commands().unwrap_err().description,
            QueryError::from("Invalid remove query").description
        );
    }
}
