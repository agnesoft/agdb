use super::remove_index_id::RemoveIndexId;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::DbElement;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertIndex {}

impl InsertIndex {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &mut Context,
    ) -> Result<CommandsMut, QueryError> {
        db.indexes.insert(&context.id, &context.graph_index)?;
        db.next_index += 1;
        result.result += 1;
        result.elements.push(DbElement {
            index: context.id,
            values: vec![],
        });

        Ok(CommandsMut::RemoveIndexId(RemoveIndexId { id: context.id }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", InsertIndex {});
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(InsertIndex {}, InsertIndex {});
    }
}
