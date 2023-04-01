use super::remove_node::RemoveNode;
use super::CommandsMut;
use crate::query::query_id::QueryId;
use crate::Db;
use crate::DbElement;
use crate::QueryError;
use crate::QueryResult;

#[derive(Debug, PartialEq)]
pub struct InsertNode {
    pub(crate) alias: Option<String>,
}

impl InsertNode {
    pub(crate) fn process(
        &self,
        db: &mut Db,
        result: &mut QueryResult,
    ) -> Result<CommandsMut, QueryError> {
        let index = self.insert_node_write_data(db)?;

        result.result += 1;
        result.elements.push(DbElement {
            index,
            values: vec![],
        });

        Ok(CommandsMut::RemoveNode(RemoveNode {
            id: QueryId::Id(index),
        }))
    }

    fn insert_node_write_data(&self, db: &mut Db) -> Result<i64, QueryError> {
        let index = db.next_node;
        let graph_index = db.graph.insert_node()?.index;
        db.next_node += 1;

        if let Some(alias) = &self.alias {
            db.aliases.insert(alias, &index)?
        }

        db.indexes.insert(&index, &graph_index)?;

        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", InsertNode { alias: None });
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(InsertNode { alias: None }, InsertNode { alias: None });
    }
}
