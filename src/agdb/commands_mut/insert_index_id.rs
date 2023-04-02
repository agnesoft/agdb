use super::remove_index_id::RemoveIndexId;
use super::CommandsMut;
use crate::graph::graph_index::GraphIndex;
use crate::Db;
use crate::DbId;
use crate::QueryError;

#[derive(Debug, PartialEq)]
pub struct InsertIndexId {
    pub(crate) id: DbId,
    pub(crate) graph_index: GraphIndex,
}

impl InsertIndexId {
    pub(crate) fn process(&self, db: &mut Db) -> Result<CommandsMut, QueryError> {
        db.indexes.insert(&self.id, &self.graph_index)?;

        Ok(CommandsMut::RemoveIndexId(RemoveIndexId { id: self.id }))
    }
}
