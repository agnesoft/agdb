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
use crate::collections::indexed_map::IndexedMap;
use crate::commands::Commands;
use crate::graph::graph_index::GraphIndex;
use crate::query::query_id::QueryId;
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
        let mut context = Context {
            index: 0,
            from: 0,
            to: 0,
        };
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
            Commands::InsertEdge(data) => {
                let from;
                let to;

                {
                    let db_data = self.data.read()?;
                    from = Self::graph_index_from_query_id(
                        &data.from,
                        &db_data.aliases,
                        &db_data.indexes,
                    )?;
                    to = Self::graph_index_from_query_id(
                        &data.to,
                        &db_data.aliases,
                        &db_data.indexes,
                    )?;
                    context.index = db_data.next_edge;
                }

                {
                    let mut mut_data = self.data.write()?;
                    let graph_index = mut_data.graph.insert_edge(&from, &to)?.index;
                    mut_data.next_edge += 1;
                    mut_data.indexes.insert(&context.index, &graph_index)?;
                }

                result.result += 1;
                result.elements.push(crate::DbElement {
                    index: context.index,
                    values: vec![],
                });
            }
            Commands::InsertNode(_) => {
                {
                    let mut mut_data = self.data.write()?;
                    context.index = mut_data.next_node;
                    let graph_index = mut_data.graph.insert_node()?.index;
                    mut_data.next_node += 1;
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

    fn graph_index_from_query_id(
        id: &QueryId,
        aliases: &IndexedMap<String, i64>,
        indexes: &IndexedMap<i64, i64>,
    ) -> Result<GraphIndex, QueryError> {
        Ok(match id {
            QueryId::Id(id) => GraphIndex::from(
                indexes
                    .value(id)?
                    .ok_or(QueryError::from(format!("Id '{id}' not found")))?,
            ),
            QueryId::Alias(alias) => GraphIndex::from(
                aliases
                    .value(alias)?
                    .ok_or(QueryError::from(format!("Alias '{alias}' not found")))?,
            ),
        })
    }
}
