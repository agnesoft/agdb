use super::remove_edge::RemoveEdge;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::graph::graph_index::GraphIndex;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::DbElement;
use crate::QueryError;
use crate::QueryResult;

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
        result: &mut QueryResult,
    ) -> Result<CommandsMut, QueryError> {
        let (from, to) = self.get_from_to(db, context)?;
        Self::insert_edge_write_data(db, from, to, context)?;

        result.result += 1;
        result.elements.push(DbElement {
            index: context.index,
            values: vec![],
        });

        Ok(CommandsMut::RemoveEdge(RemoveEdge {
            id: QueryId::Id(context.index),
        }))
    }

    fn get_from_to(
        &self,
        db: &Db,
        context: &mut Context,
    ) -> Result<(GraphIndex, GraphIndex), QueryError> {
        let from = db.graph_index_from_id(&self.from)?;
        let to = db.graph_index_from_id(&self.to)?;
        context.index = db.next_edge;

        Ok((from, to))
    }

    fn insert_edge_write_data(
        db: &mut Db,
        from: GraphIndex,
        to: GraphIndex,
        context: &mut Context,
    ) -> Result<(), QueryError> {
        let graph_index = db.graph.insert_edge(&from, &to)?.index;
        db.next_edge -= 1;
        Ok(db.indexes.insert(&context.index, &graph_index)?)
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
                from: QueryId::Id(0),
                to: QueryId::Id(0)
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertEdge {
                from: QueryId::Id(0),
                to: QueryId::Id(0)
            },
            InsertEdge {
                from: QueryId::Id(0),
                to: QueryId::Id(0)
            }
        );
    }
}
