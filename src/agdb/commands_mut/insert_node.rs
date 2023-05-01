use crate::graph::graph_index::GraphIndex;
use crate::Db;
use crate::DbElement;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertNode {
    id: DbId,
    graph_index: GraphIndex,
    alias: String,
}

impl InsertNode {
    pub(crate) fn new(alias: String) -> Self {
        Self {
            id: DbId(0),
            graph_index: GraphIndex { index: 0 },
            alias,
        }
    }

    pub(crate) fn redo(&mut self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        self.graph_index = db.graph.insert_node()?;
        self.id = DbId(db.next_id);
        db.next_id += 1;
        db.indexes.insert(&self.id, &self.graph_index)?;

        if !self.alias.is_empty() {
            db.aliases.insert(&self.alias, &self.id)?;
        }

        result.result += 1;
        result.elements.push(DbElement {
            index: self.id,
            values: vec![],
        });

        Ok(())
    }

    pub(crate) fn undo(self, db: &mut Db) -> Result<(), QueryError> {
        db.graph.remove_node(&self.graph_index)?;
        db.indexes.remove_key(&self.id)?;
        db.aliases.remove_key(&self.alias)?;
        db.next_id -= 1;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", InsertNode::new(String::new()));
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertNode::new(String::new()),
            InsertNode::new(String::new())
        );
    }
}
