use super::insert_node::InsertNode;
use super::CommandsMut;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct RemoveNode {
    pub(crate) id: QueryId,
}

impl RemoveNode {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
    ) -> Result<CommandsMut, QueryError> {
        let index = db.graph_index_from_id(&self.id)?;

        let alias = match &self.id {
            QueryId::Id(id) => db.aliases.key(id)?,
            QueryId::Alias(alias) => Some(alias.clone()),
        };

        if let Some(alias) = &alias {
            db.aliases.remove_key(alias)?;
        }

        db.graph.remove_node(&index)?;
        db.indexes.remove_value(&index.index)?;
        result.result -= 1;
        Ok(CommandsMut::InsertNode(InsertNode { alias }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveNode { id: QueryId::Id(0) });
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveNode { id: QueryId::Id(0) },
            RemoveNode { id: QueryId::Id(0) }
        );
    }
}
