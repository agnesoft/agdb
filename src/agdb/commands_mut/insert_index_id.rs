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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            InsertIndexId {
                id: DbId(0),
                graph_index: GraphIndex { index: 0 }
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertIndexId {
                id: DbId(0),
                graph_index: GraphIndex { index: 0 }
            },
            InsertIndexId {
                id: DbId(0),
                graph_index: GraphIndex { index: 0 }
            }
        );
    }
}
