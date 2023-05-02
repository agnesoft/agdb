use crate::graph::graph_index::GraphIndex;
use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveEdge {
    id: DbId,
    graph_index: GraphIndex,
    from: GraphIndex,
    to: GraphIndex,
}

impl RemoveEdge {
    pub(crate) fn new(id: DbId) -> Self {
        Self {
            id,
            graph_index: GraphIndex::new(),
            from: GraphIndex::new(),
            to: GraphIndex::new(),
        }
    }

    pub(crate) fn redo(&mut self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        if let Some(graph_index) = db.indexes.value(&self.id)? {
            let edge = db
                .graph
                .edge(&graph_index)
                .ok_or(QueryError::from("Invalid graph id"))?;
            self.from = edge.index_from();
            self.to = edge.index_to();
            db.graph.remove_edge(&graph_index)?;
            self.graph_index = graph_index;
            db.indexes.remove_key(&self.id)?;
            result.result -= 1;
        }

        Ok(())
    }

    pub(crate) fn undo(self, db: &mut Db) -> Result<(), QueryError> {
        if self.graph_index.is_valid() {
            let graph_index = db.graph.insert_edge(&self.from, &self.to)?;
            db.indexes.insert(&self.id, &graph_index)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveEdge::new(DbId(0)));
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(RemoveEdge::new(DbId(0)), RemoveEdge::new(DbId(0)));
    }
}
