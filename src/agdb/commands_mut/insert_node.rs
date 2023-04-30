use super::remove_node::RemoveNode;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::graph::graph_index::GraphIndex;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct InsertNode {
    graph_index: GraphIndex,
}

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

    pub(crate) fn new() -> InsertNode {
        InsertNode {
            graph_index: GraphIndex { index: 0 },
        }
    }

    pub(crate) fn redo(&mut self, db: &mut Db, context: &mut Context) -> Result<(), QueryError> {
        self.graph_index = db.graph.insert_node()?;
        context.graph_index = self.graph_index;
        Ok(())
    }

    pub(crate) fn undo(&mut self, db: &mut Db) -> Result<(), QueryError> {
        Ok(db.graph.remove_node(&self.graph_index)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", InsertNode::new());
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(InsertNode::new(), InsertNode::new());
    }
}
