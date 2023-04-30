use crate::db::db_context::Context;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct RemoveNode {}

impl RemoveNode {
    pub(crate) fn new() -> Self {
        Self {}
    }

    pub(crate) fn redo(&mut self, db: &mut Db, context: &mut Context) -> Result<(), QueryError> {
        Ok(db.graph.remove_node(&context.graph_index)?)
    }

    pub(crate) fn undo(&mut self, db: &mut Db) -> Result<(), QueryError> {
        db.graph.insert_node()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveNode::new());
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(RemoveNode::new(), RemoveNode::new());
    }
}
