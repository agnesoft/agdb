use crate::graph::graph_index::GraphIndex;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveNode {
    id: DbId,
    alias: String,
    graph_index: GraphIndex,
}

impl RemoveNode {
    pub(crate) fn new(id: QueryId) -> Self {
        match id {
            QueryId::Id(id) => Self {
                id,
                alias: String::new(),
                graph_index: GraphIndex { index: 0 },
            },
            QueryId::Alias(alias) => Self {
                id: DbId(0),
                alias,
                graph_index: GraphIndex { index: 0 },
            },
        }
    }

    pub(crate) fn redo(&mut self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        if !self.alias.is_empty() {
            self.id = db.aliases.value(&self.alias)?.unwrap_or_default();
        } else {
            self.alias = db.aliases.key(&self.id)?.unwrap_or_default();
        }

        if let Some(graph_index) = db.indexes.value(&self.id)? {
            db.graph.remove_node(&graph_index)?;
            self.graph_index = graph_index;
            db.aliases.remove_key(&self.alias)?;
            db.indexes.remove_key(&self.id)?;
            result.result -= 1;
        }

        Ok(())
    }

    pub(crate) fn undo(self, db: &mut Db) -> Result<(), QueryError> {
        if self.graph_index.is_valid() {
            let graph_index = db.graph.insert_node()?;
            db.indexes.insert(&self.id, &graph_index)?;

            if !self.alias.is_empty() {
                db.aliases.insert(&self.alias, &self.id)?;
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
        format!("{:?}", RemoveNode::new("alias".into()));
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveNode::new("alias".into()),
            RemoveNode::new("alias".into())
        );
    }
}
