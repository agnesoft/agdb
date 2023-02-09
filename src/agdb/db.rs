pub mod db_element;
pub mod db_error;
pub mod db_index;
pub mod db_key;
pub mod db_key_value;
pub mod db_value;

mod db_context;
mod db_data;
mod db_float;

use self::db_context::Context;
use self::db_data::DbData;
use crate::commands::Commands;
use crate::query::Query;
use crate::DbError;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;
use std::sync::RwLock;

pub struct Db {
    data: RwLock<DbData>,
}

impl Db {
    pub fn exec(&self, query: &Query) -> Result<QueryResult, QueryError> {
        let mut context = Context { index: 0 };
        let commands = query.commands();
        let mut result = QueryResult {
            result: 0,
            elements: vec![],
        };

        for command in commands {
            self.process_command(command, &mut context, &mut result)?;
        }

        Ok(result)
    }

    pub fn new(_filename: &str) -> Result<Db, DbError> {
        Ok(Self {
            data: RwLock::new(DbData::new()?),
        })
    }

    pub fn transaction(&self) -> Transaction {
        Transaction::default()
    }

    fn process_command(
        &self,
        command: Commands,
        context: &mut Context,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        match command {
            Commands::InsertAlias(data) => {
                self.data
                    .write()?
                    .aliases
                    .insert(&data.alias, &context.index)?;
            }
            Commands::InsertEdge(_) => todo!(),
            Commands::InsertNode(_) => {
                {
                    let mut mut_data = self.data.write()?;
                    context.index = mut_data.next_node;
                    mut_data.next_node += 1;
                    let graph_index = mut_data.graph.insert_node()?.index;
                    mut_data.indexes.insert(&context.index, &graph_index)?;
                }

                result.result += 1;
                result.elements.push(crate::DbElement {
                    index: context.index,
                    values: vec![],
                });
            }
            Commands::RemoveAlias(_) => todo!(),
            Commands::RemoveEdge(_) => todo!(),
            Commands::RemoveNode(_) => todo!(),
        }

        Ok(())
    }
}
