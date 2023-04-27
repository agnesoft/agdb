use super::insert_index::InsertIndex;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveIndex {}

impl RemoveIndex {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &Context,
    ) -> Result<CommandsMut, QueryError> {
        if let Some(graph_index) = db.indexes.value(&context.id)? {
            db.indexes.remove_key(&context.id)?;
            result.result -= 1;

            Ok(CommandsMut::InsertIndex(InsertIndex {
                id: Some(context.id),
                graph_index: Some(graph_index),
            }))
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
        format!("{:?}", RemoveIndex {});
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(RemoveIndex {}, RemoveIndex {});
    }
}
