use crate::graph::graph_index::GraphIndex;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::DbElement;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertEdge {
    id: DbId,
    graph_index: GraphIndex,
    from: QueryId,
    to: QueryId,
}

impl InsertEdge {
    pub(crate) fn new(from: QueryId, to: QueryId) -> Self {
        Self {
            id: DbId(0),
            graph_index: GraphIndex::new(),
            from,
            to,
        }
    }

    pub(crate) fn redo(&mut self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        let from = db.graph_index_from_id(&self.from)?;
        let to = db.graph_index_from_id(&self.to)?;
        self.graph_index = db.graph.insert_edge(&from, &to)?;
        self.id = DbId(-db.next_id);
        db.next_id += 1;
        db.indexes.insert(&self.id, &self.graph_index)?;
        result.result += 1;
        result.elements.push(DbElement {
            index: self.id,
            values: vec![],
        });

        Ok(())
    }

    pub(crate) fn undo(self, db: &mut Db) -> Result<(), QueryError> {
        db.graph.remove_edge(&self.graph_index)?;
        db.indexes.remove_key(&self.id)?;
        db.next_id -= 1;

        Ok(())
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
