use super::insert_node::InsertNode;
use super::CommandsMut;
use crate::db::db_context::Context;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveNode {}

impl RemoveNode {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
        context: &Context,
    ) -> Result<CommandsMut, QueryError> {
        db.graph.remove_node(&context.graph_index)?;
        result.result -= 1;

        Ok(CommandsMut::InsertNode(InsertNode {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveNode {});
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(RemoveNode {}, RemoveNode {});
    }
}
