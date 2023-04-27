use super::insert_index::InsertIndex;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveIndexId {
    pub(crate) id: DbId,
}

impl RemoveIndexId {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &mut Context,
    ) -> Result<CommandsMut, QueryError> {
        if let Some(graph_index) = db.indexes.value(&self.id)? {
            context.graph_index = graph_index;
            db.indexes.remove_key(&self.id)?;
            result.result -= 1;

            Ok(CommandsMut::InsertIndex(InsertIndex {
                id: Some(self.id),
                graph_index: Some(context.graph_index),
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
        format!("{:?}", RemoveIndexId { id: DbId(0) });
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(RemoveIndexId { id: DbId(0) }, RemoveIndexId { id: DbId(0) });
    }
}
