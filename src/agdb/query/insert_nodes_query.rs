use super::query_values::QueryValues;
use super::Query;
use super::QueryMut;
use crate::commands::insert_node::InsertNode;
use crate::commands::Commands;
use crate::QueryError;

pub struct InsertNodesQuery {
    pub count: u64,
    pub values: QueryValues,
    pub aliases: Vec<String>,
}

impl Query for InsertNodesQuery {
    fn commands(&self) -> Result<Vec<Commands>, QueryError> {
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

        Ok(commands)
    }
}

impl QueryMut for InsertNodesQuery {}
