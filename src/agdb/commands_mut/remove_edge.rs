use super::insert_edge::InsertEdge;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct RemoveEdge {}

impl RemoveEdge {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        context: &Context,
    ) -> Result<CommandsMut, QueryError> {
        if let Some(edge) = db.graph.edge(&context.graph_index) {
            let undo = CommandsMut::InsertEdge(InsertEdge {
                from: QueryId::Id(edge.index_from().index),
                to: QueryId::Id(edge.index_to().index),
            });
            db.graph.remove_edge(&context.graph_index)?;

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
        format!("{:?}", RemoveEdge {});
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(RemoveEdge {}, RemoveEdge {});
    }
}
