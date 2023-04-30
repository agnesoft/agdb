use crate::db::db_context::Context;
use crate::graph::graph_index::GraphIndex;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct RemoveEdge {
    from: GraphIndex,
    to: GraphIndex,
}

impl RemoveEdge {
    pub(crate) fn new() -> Self {
        Self {
            from: GraphIndex { index: 0 },
            to: GraphIndex { index: 0 },
        }
    }

    pub(crate) fn redo(&mut self, db: &mut Db, context: &mut Context) -> Result<(), QueryError> {
        if let Some(edge) = db.graph.edge(&context.graph_index) {
            self.from = edge.index_from();
            self.to = edge.index_to();
            db.graph.remove_edge(&context.graph_index)?;
        }

        Ok(())
    }

    pub(crate) fn undo(&mut self, db: &mut Db) -> Result<(), QueryError> {
        db.graph.insert_edge(&self.from, &self.to)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveEdge::new());
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(RemoveEdge::new(), RemoveEdge::new());
    }
}
