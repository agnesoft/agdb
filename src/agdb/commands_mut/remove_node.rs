use crate::db::db_context::Context;
use crate::graph::graph_index::GraphIndex;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct RemoveNode {
    pub(crate) index: Option<GraphIndex>,
}

impl RemoveNode {
    pub(crate) fn redo(&mut self, db: &mut Db, context: &mut Context) -> Result<(), QueryError> {
        todo!()
    }

    pub(crate) fn undo(&mut self, db: &mut Db) -> Result<(), QueryError> {
        todo!()
    }

    // pub(crate) fn process(
    //     &self,
    //     db: &mut Db,
    //     context: &Context,
    // ) -> Result<CommandsMut, QueryError> {
    //     let index = self.index.unwrap_or(context.graph_index);
    //     db.graph.remove_node(&index)?;
    //     Ok(CommandsMut::InsertNode(InsertNode::new()))
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveNode { index: None });
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(RemoveNode { index: None }, RemoveNode { index: None });
    }
}
