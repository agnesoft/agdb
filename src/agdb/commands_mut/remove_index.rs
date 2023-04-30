use super::insert_index::InsertIndex;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveIndex {
    pub(crate) id: Option<DbId>,
}

impl RemoveIndex {
    pub(crate) fn redo(
        &mut self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &mut Context,
    ) -> Result<(), QueryError> {
        todo!()
    }

    pub(crate) fn undo(&mut self, db: &mut Db) -> Result<(), QueryError> {
        todo!()
    }

    // pub(crate) fn process(
    //     &self,
    //     db: &mut Db,
    //     result: &mut QueryResult,
    //     context: &mut Context,
    // ) -> Result<CommandsMut, QueryError> {
    //     let id = self.id.unwrap_or(context.id);

    //     if let Some(graph_index) = db.indexes.value(&id)? {
    //         context.graph_index = graph_index;
    //         db.indexes.remove_key(&id)?;
    //         result.result -= 1;
    //         Ok(CommandsMut::InsertIndex(InsertIndex {
    //             id: Some(id),
    //             graph_index: Some(graph_index),
    //         }))
    //     } else {
    //         Ok(CommandsMut::None)
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveIndex { id: None });
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(RemoveIndex { id: None }, RemoveIndex { id: None });
    }
}
