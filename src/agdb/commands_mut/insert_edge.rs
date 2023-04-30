use crate::db::db_context::Context;
use crate::graph::graph_index::GraphIndex;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct InsertEdge {
    graph_index: GraphIndex,
    from: QueryId,
    to: QueryId,
}

impl InsertEdge {
    pub(crate) fn new(from: QueryId, to: QueryId) -> Self {
        Self {
            graph_index: GraphIndex { index: 0 },
            from,
            to,
        }
    }

    pub(crate) fn redo(&mut self, db: &mut Db, context: &mut Context) -> Result<(), QueryError> {
        let from = db.graph_index_from_id(&self.from)?;
        let to = db.graph_index_from_id(&self.to)?;
        self.graph_index = db.graph.insert_edge(&from, &to)?;
        context.graph_index = self.graph_index;
        Ok(())
    }

    pub(crate) fn undo(&mut self, db: &mut Db) -> Result<(), QueryError> {
        Ok(db.graph.remove_edge(&self.graph_index)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", InsertEdge::new(QueryId::from(0), QueryId::from(0)));
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertEdge::new(QueryId::from(0), QueryId::from(0)),
            InsertEdge::new(QueryId::from(0), QueryId::from(0))
        );
    }
}
