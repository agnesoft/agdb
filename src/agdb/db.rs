pub mod db_element;
pub mod db_error;
pub mod db_index;
pub mod db_key;
pub mod db_key_value;
pub mod db_value;

mod db_float;

use crate::commands::Commands;
use crate::db_data::DbData;
use crate::graph::graph_index::GraphIndex;
use crate::query::Query;
use crate::DbError;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;
use std::sync::RwLock;

pub struct Db {
    data: RwLock<DbData>,
}

struct Context {
    pub index: GraphIndex,
}

impl Db {
    pub fn exec(&self, query: &Query) -> Result<QueryResult, QueryError> {
        let mut context = Context {
            index: GraphIndex::default(),
        };
        let commands = query.commands();
        let mut result = QueryResult {
            result: 0,
            elements: vec![],
        };

        for command in commands {
            match command {
                Commands::InsertAlias(data) => {
                    self.data
                        .write()?
                        .aliases
                        .insert(data.alias, context.index.clone());
                }
                Commands::InsertEdge(_) => todo!(),
                Commands::InsertNode(_) => {
                    context.index = self.data.write()?.graph.insert_node()?;
                    result.result += 1;
                    result.elements.push(crate::DbElement {
                        index: context.index.as_u64(),
                        values: vec![],
                    });
                }
                Commands::RemoveAlias(_) => todo!(),
                Commands::RemoveEdge(_) => todo!(),
                Commands::RemoveNode(_) => todo!(),
            }
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
}
