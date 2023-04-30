use super::remove_index::RemoveIndex;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::graph::graph_index::GraphIndex;
use crate::Db;
use crate::DbElement;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertIndex {
    pub(crate) id: Option<DbId>,
    pub(crate) graph_index: Option<GraphIndex>,
}

impl InsertIndex {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &mut Context,
    ) -> Result<CommandsMut, QueryError> {
        let graph_index = self.graph_index.unwrap_or(context.graph_index);

        let id = if let Some(id) = self.id {
            id
        } else {
            context.id = if graph_index.is_node() {
                DbId(db.next_index)
            } else {
                DbId(-db.next_index)
            };

            db.next_index += 1;
            context.id
        };

        db.indexes.insert(&id, &graph_index)?;
        result.result += 1;
        result.elements.push(DbElement {
            index: context.id,
            values: vec![],
        });

        Ok(CommandsMut::RemoveIndex(RemoveIndex { id: Some(id) }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            InsertIndex {
                id: Some(DbId(0)),
                graph_index: Some(GraphIndex { index: 0 })
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertIndex {
                id: Some(DbId(0)),
                graph_index: Some(GraphIndex { index: 0 })
            },
            InsertIndex {
                id: Some(DbId(0)),
                graph_index: Some(GraphIndex { index: 0 })
            }
        );
    }
}
