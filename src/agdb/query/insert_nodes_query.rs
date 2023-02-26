use super::query_values::QueryValues;
use crate::commands::insert_node::InsertNode;
use crate::commands::Commands;

pub struct InsertNodesQuery {
    pub count: u64,
    pub values: QueryValues,
    pub aliases: Vec<String>,
}

impl InsertNodesQuery {
    pub(crate) fn commands(&self) -> Vec<Commands> {
        let mut commands = Vec::<Commands>::new();

        if self.aliases.is_empty() {
            for _i in 0..self.count {
                commands.push(Commands::InsertNode(InsertNode { alias: None }));
            }
        } else {
            for alias in &self.aliases {
                commands.push(Commands::InsertNode(InsertNode {
                    alias: Some(alias.clone()),
                }));
            }
        }

        commands
    }
}
