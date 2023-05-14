use super::query_values::QueryValues;
use super::QueryMut;
use crate::commands_mut::insert_node::InsertNode;
use crate::commands_mut::CommandsMut;
use crate::Db;
use crate::DbElement;
use crate::QueryError;
use crate::QueryResult;

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
                commands.push(CommandsMut::InsertNode(InsertNode::new(String::new())));
            }
        } else {
            for alias in &self.aliases {
                commands.push(CommandsMut::InsertNode(InsertNode::new(alias.clone())));
            }
        }

        Ok(commands)
    }

    fn process(&self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        let mut ids = vec![];

        if self.aliases.is_empty() {
            for _i in 0..self.count {
                ids.push(db.insert_node()?);
            }
        } else {
            for alias in &self.aliases {
                let db_id: crate::DbId = db.insert_node()?;
                db.insert_new_alias(db_id, alias)?;
                ids.push(db_id);
            }
        }

        result.result = ids.len() as i64;
        result.elements = ids
            .into_iter()
            .map(|id| DbElement {
                index: id,
                values: vec![],
            })
            .collect();

        Ok(())
    }
}
