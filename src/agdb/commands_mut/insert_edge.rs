use super::remove_edge::RemoveEdge;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::graph::graph_index::GraphIndex;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct InsertEdge {
    pub(crate) from: QueryId,
    pub(crate) to: QueryId,
}

impl InsertEdge {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        context: &mut Context,
    ) -> Result<CommandsMut, QueryError> {
        let (from, to) = self.get_from_to(db)?;
        context.graph_index = db.graph.insert_edge(&from, &to)?;

        Ok(CommandsMut::RemoveEdge(RemoveEdge {
            index: Some(context.graph_index),
        }))
    }

    fn get_from_to(&self, db: &Db) -> Result<(GraphIndex, GraphIndex), QueryError> {
        let from = db.graph_index_from_id(&self.from)?;
        let to = db.graph_index_from_id(&self.to)?;
        Ok((from, to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            InsertEdge {
                from: QueryId::from(0),
                to: QueryId::from(0)
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertEdge {
                from: QueryId::from(0),
                to: QueryId::from(0)
            },
            InsertEdge {
                from: QueryId::from(0),
                to: QueryId::from(0)
            }
        );
    }
}
