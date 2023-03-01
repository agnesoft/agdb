use super::query_id::QueryId;
use super::query_ids::QueryIds;
use crate::commands::remove_edge::RemoveEdge;
use crate::commands::remove_node::RemoveNode;
use crate::commands::Commands;
use crate::QueryError;

pub struct RemoveQuery(pub QueryIds);

impl RemoveQuery {
    pub(crate) fn commands(&self) -> Result<Vec<Commands>, QueryError> {
        match &self.0 {
            QueryIds::Id(id) => Ok(Self::id(id)),
            QueryIds::Ids(ids) => Ok(Self::ids(ids)),
            QueryIds::All | QueryIds::Search(_) => Err(QueryError::from("Invalid remove query")),
        }
    }

    fn id(id: &QueryId) -> Vec<Commands> {
        if id.is_node() {
            vec![Commands::RemoveNode(RemoveNode { id: id.clone() })]
        } else {
            vec![Commands::RemoveEdge(RemoveEdge { id: id.clone() })]
        }
    }

    fn ids(ids: &Vec<QueryId>) -> Vec<Commands> {
        let mut commands = Vec::<Commands>::new();

        for id in ids {
            if id.is_node() {
                commands.push(Commands::RemoveNode(RemoveNode { id: id.clone() }));
            } else {
                commands.push(Commands::RemoveEdge(RemoveEdge { id: id.clone() }));
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
            Ok(vec![Commands::RemoveNode(RemoveNode {
                id: QueryId::Id(1)
            })])
        )
    }

    #[test]
    fn valid_ids() {
        let query = RemoveQuery(QueryIds::Ids(vec![QueryId::Id(1)]));

        assert_eq!(
            query.commands(),
            Ok(vec![Commands::RemoveNode(RemoveNode {
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
