use super::query_id::QueryId;
use super::query_ids::QueryIds;
use crate::commands::remove_edge::RemoveEdge;
use crate::commands::remove_node::RemoveNode;
use crate::commands::Commands;

pub struct RemoveQuery(pub QueryIds);

impl RemoveQuery {
    pub(crate) fn commands(&self) -> Vec<Commands> {
        match &self.0 {
            QueryIds::All | QueryIds::Search(_) => panic!("Invalid query"),
            QueryIds::Id(id) => Self::id(id),
            QueryIds::Ids(ids) => Self::ids(ids),
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
    use crate::test_utilities::catch_unwidn_silent::catch_unwind_silent;

    #[test]
    fn invalid_query_preprocessing_many_many() {
        let result = catch_unwind_silent(|| {
            let query = RemoveQuery(QueryIds::All);
            query.commands();
        });

        assert_eq!(
            *result.unwrap_err().downcast_ref::<&str>().unwrap(),
            "Invalid query"
        );
    }
}
