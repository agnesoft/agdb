use super::insert_index_id::InsertIndexId;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct RemoveIndex {}

impl RemoveIndex {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        context: &Context,
    ) -> Result<CommandsMut, QueryError> {
        let index = db
            .indexes
            .value(&context.id)?
            .ok_or(QueryError::from(format!(
                "Id '{}' not found",
                &context.id.id
            )))?;

        db.indexes.remove_key(&context.id)?;

        Ok(CommandsMut::InsertIndexId(InsertIndexId {
            id: context.id,
            graph_index: index,
        }))
    }
}
