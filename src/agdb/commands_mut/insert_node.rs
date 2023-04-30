use super::remove_node::RemoveNode;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct InsertNode {}

impl InsertNode {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        context: &mut Context,
    ) -> Result<CommandsMut, QueryError> {
        context.graph_index = db.graph.insert_node()?;

        Ok(CommandsMut::RemoveNode(RemoveNode {
            index: Some(context.graph_index),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", InsertNode {});
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(InsertNode {}, InsertNode {});
    }
}
