use crate::db::db_context::Context;
use crate::graph::graph_index::GraphIndex;
use crate::Db;
use crate::DbElement;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertIndex {
    id: DbId,
    graph_index: GraphIndex,
}

impl InsertIndex {
    pub(crate) fn new() -> Self {
        Self {
            id: DbId::default(),
            graph_index: GraphIndex { index: 0 },
        }
    }

    pub(crate) fn redo(
        &mut self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &mut Context,
    ) -> Result<(), QueryError> {
        self.graph_index = context.graph_index;
        self.id = if context.graph_index.is_node() {
            DbId(db.next_index)
        } else {
            DbId(-db.next_index)
        };
        context.id = self.id;
        db.next_index += 1;
        db.indexes.insert(&self.id, &self.graph_index)?;
        result.result += 1;
        result.elements.push(DbElement {
            index: self.id,
            values: vec![],
        });
        Ok(())
    }

    pub(crate) fn undo(self, db: &mut Db) -> Result<(), QueryError> {
        Ok(db.indexes.remove_key(&self.id)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", InsertIndex::new());
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(InsertIndex::new(), InsertIndex::new());
    }
}
