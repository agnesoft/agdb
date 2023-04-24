use super::insert_edge::InsertEdge;
use super::CommandsMut;
use crate::graph::graph_index::GraphIndex;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct RemoveEdgeIndex {
    pub(crate) index: GraphIndex,
}

impl RemoveEdgeIndex {
    pub(crate) fn process(&self, db: &mut Db) -> Result<CommandsMut, QueryError> {
        let edge = db.graph.edge(&self.index).ok_or(QueryError::from(format!(
            "Index '{}' not found",
            self.index.index
        )))?;
        let undo = CommandsMut::InsertEdge(InsertEdge {
            from: QueryId::from(edge.index_from().index),
            to: QueryId::from(edge.index_to().index),
        });
        db.graph.remove_edge(&self.index)?;

        Ok(undo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            RemoveEdgeIndex {
                index: GraphIndex { index: 0 }
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveEdgeIndex {
                index: GraphIndex { index: 0 }
            },
            RemoveEdgeIndex {
                index: GraphIndex { index: 0 }
            }
        );
    }
}
