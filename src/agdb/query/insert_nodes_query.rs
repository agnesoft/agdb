use super::query_values::QueryValues;
use super::QueryMut;
use crate::commands_mut::insert_alias::InsertAlias;
use crate::commands_mut::insert_index::InsertIndex;
use crate::commands_mut::insert_node::InsertNode;
use crate::commands_mut::CommandsMut;
use crate::QueryError;

pub struct InsertNodesQuery {
    pub count: u64,
    pub values: QueryValues,
    pub aliases: Vec<String>,
}

impl QueryMut for InsertNodesQuery {
    fn commands(&self) -> Result<Vec<CommandsMut>, QueryError> {
        let mut commands = Vec::<CommandsMut>::new();

        if self.aliases.is_empty() {
            for _i in 0..self.count {
                commands.push(CommandsMut::InsertNode(InsertNode::new()));
                commands.push(CommandsMut::InsertIndex(InsertIndex::new()));
            }
        } else {
            for alias in &self.aliases {
                commands.push(CommandsMut::InsertNode(InsertNode::new()));
                commands.push(CommandsMut::InsertIndex(InsertIndex::new()));
                commands.push(CommandsMut::InsertAlias(InsertAlias::new(
                    alias.clone(),
                    None,
                )));
            }
        }

        Ok(commands)
    }
}
