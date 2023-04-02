use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::QueryMut;
use crate::commands_mut::remove_alias::RemoveAlias;
use crate::commands_mut::remove_edge::RemoveEdge;
use crate::commands_mut::remove_index::RemoveIndex;
use crate::commands_mut::remove_index_id::RemoveIndexId;
use crate::commands_mut::remove_node::RemoveNode;
use crate::commands_mut::CommandsMut;
use crate::DbId;
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
        let mut commands = Self::remove_index(id);

        if id.is_node() {
            commands.push(CommandsMut::RemoveNode(RemoveNode {}));
        } else {
            commands.push(CommandsMut::RemoveEdge(RemoveEdge {}));
        }

        commands
    }

    fn ids(ids: &Vec<QueryId>) -> Vec<CommandsMut> {
        let mut commands = Vec::<CommandsMut>::new();

        for id in ids {
            commands.extend(Self::id(id));
        }

        commands
    }

    fn remove_index(id: &QueryId) -> Vec<CommandsMut> {
        let commands = match id {
            QueryId::Id(id) => vec![CommandsMut::RemoveIndexId(RemoveIndexId {
                id: DbId { id: id.clone() },
            })],
            QueryId::Alias(alias) => vec![
                CommandsMut::RemoveAlias(RemoveAlias {
                    alias: alias.clone(),
                }),
                CommandsMut::RemoveIndex(RemoveIndex {}),
            ],
        };

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
            Ok(vec![CommandsMut::RemoveNode(RemoveNode {})])
        )
    }

    #[test]
    fn valid_ids() {
        let query = RemoveQuery(QueryIds::Ids(vec![QueryId::Id(1)]));

        assert_eq!(
            query.commands(),
            Ok(vec![CommandsMut::RemoveNode(RemoveNode {})])
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
