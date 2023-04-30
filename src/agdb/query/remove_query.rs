use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::QueryMut;
use crate::commands_mut::remove_alias::RemoveAlias;
use crate::commands_mut::remove_edge::RemoveEdge;
use crate::commands_mut::remove_index::RemoveIndex;
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
    fn id(id: &QueryId) -> Vec<CommandsMut> {
        let mut commands = Self::remove_index(id);

        if id.is_node() {
            if let QueryId::Id(id) = id {
                commands.push(CommandsMut::RemoveAlias(RemoveAlias::new_id(*id)));
            }

            commands.push(CommandsMut::RemoveNode(RemoveNode { index: None }));
        } else {
            commands.push(CommandsMut::RemoveEdge(RemoveEdge::new()));
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
        match id {
            QueryId::Id(id) => vec![CommandsMut::RemoveIndex(RemoveIndex::new(Some(*id)))],
            QueryId::Alias(alias) => vec![
                CommandsMut::RemoveAlias(RemoveAlias::new(alias.clone())),
                CommandsMut::RemoveIndex(RemoveIndex::new(None)),
            ],
        }
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
            Ok(vec![
                CommandsMut::RemoveIndex(RemoveIndex::new(Some(DbId(1)))),
                CommandsMut::RemoveAlias(RemoveAlias::new_id(DbId(1))),
                CommandsMut::RemoveNode(RemoveNode { index: None })
            ])
        )
    }

    #[test]
    fn valid_ids() {
        let query = RemoveQuery(QueryIds::Ids(vec![QueryId::from(-3)]));

        assert_eq!(
            query.commands(),
            Ok(vec![
                CommandsMut::RemoveIndex(RemoveIndex::new(Some(DbId(-3)))),
                CommandsMut::RemoveEdge(RemoveEdge::new())
            ])
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
