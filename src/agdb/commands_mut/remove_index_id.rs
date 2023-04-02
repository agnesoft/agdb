use super::insert_index_id::InsertIndexId;
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
        context.graph_index = db
            .indexes
            .value(&self.id)?
            .ok_or(QueryError::from(format!("Id '{}' not found", &self.id.id)))?;
        db.indexes.remove_key(&self.id)?;
        result.result -= 1;

        Ok(CommandsMut::InsertIndexId(InsertIndexId {
            id: self.id,
            graph_index: context.graph_index,
        }))
    }
}
