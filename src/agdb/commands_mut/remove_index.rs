use crate::db::db_context::Context;
use crate::graph::graph_index::GraphIndex;
use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveIndex {
    id: Option<DbId>,
    graph_index: GraphIndex,
}

impl RemoveIndex {
    pub(crate) fn new(id: Option<DbId>) -> Self {
        Self {
            id,
            graph_index: GraphIndex { index: 0 },
        }
    }

    pub(crate) fn redo(
        &mut self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &mut Context,
    ) -> Result<(), QueryError> {
        if let Some(id) = &self.id {
            context.id = *id;
        }

        if let Some(graph_index) = db.indexes.value(&context.id)? {
            self.id = Some(context.id);
            self.graph_index = graph_index;
            context.graph_index = graph_index;
            db.indexes.remove_key(&context.id)?;
            result.result -= 1;
        }

        Ok(())
    }

    pub(crate) fn undo(self, db: &mut Db) -> Result<(), QueryError> {
        if let Some(id) = self.id {
            if self.graph_index.is_valid() {
                db.indexes.insert(&id, &self.graph_index)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveIndex::new(None));
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(RemoveIndex::new(None), RemoveIndex::new(None));
    }
}
