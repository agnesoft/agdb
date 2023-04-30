use super::insert_edge::InsertEdge;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::graph::graph_index::GraphIndex;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct RemoveEdge {
    pub(crate) index: Option<GraphIndex>,
}

impl RemoveEdge {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        context: &Context,
    ) -> Result<CommandsMut, QueryError> {
        let graph_index = self.index.unwrap_or(context.graph_index);

        if let Some(edge) = db.graph.edge(&graph_index) {
            let undo = CommandsMut::InsertEdge(InsertEdge {
                from: QueryId::from(db.id_from_graph_index(&edge.index_from())?),
                to: QueryId::from(db.id_from_graph_index(&edge.index_to())?),
            });
            db.graph.remove_edge(&graph_index)?;

            Ok(undo)
        } else {
            Ok(CommandsMut::None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveEdge { index: None });
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(RemoveEdge { index: None }, RemoveEdge { index: None });
    }
}
