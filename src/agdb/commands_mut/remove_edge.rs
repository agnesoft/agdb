use super::insert_edge::InsertEdge;
use super::CommandsMut;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveEdge {
    pub(crate) id: QueryId,
}

impl RemoveEdge {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
    ) -> Result<CommandsMut, QueryError> {
        let index = db.graph_index_from_id(&self.id)?;
        let edge = db
            .graph
            .edge(&index)
            .ok_or(QueryError::from(format!("Id '{}' not found", index.index)))?;
        let undo = CommandsMut::InsertEdge(InsertEdge {
            from: QueryId::Id(edge.index_from().index),
            to: QueryId::Id(edge.index_to().index),
        });
        db.graph.remove_edge(&index)?;
        db.indexes.remove_value(&index.index)?;
        result.result -= 1;
        Ok(undo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveEdge { id: QueryId::Id(0) });
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveEdge { id: QueryId::Id(0) },
            RemoveEdge { id: QueryId::Id(0) }
        );
    }
}
