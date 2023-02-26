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
use crate::commands::insert_alias::InsertAlias;
use crate::commands::insert_edge::InsertEdge;
use crate::commands::insert_node::InsertNode;
use crate::commands::remove_alias::RemoveAlias;
use crate::commands::remove_edge::RemoveEdge;
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

    fn get_from_to(
        &self,
        data: InsertEdge,
        context: &mut Context,
    ) -> Result<(GraphIndex, GraphIndex), QueryError> {
        let db_data = self.data.read()?;
        let from = Self::graph_index_from_query_id(&data.from, &*self.data.read()?)?;
        let to = Self::graph_index_from_query_id(&data.to, &*self.data.read()?)?;
        context.index = db_data.next_edge;

        Ok((from, to))
    }

    fn get_index_from_id(id: &QueryId, db_data: &DbData) -> Result<i64, QueryError> {
        Ok(match id {
            QueryId::Id(id) => {
                let _ = db_data
                    .indexes
                    .value(id)?
                    .ok_or(QueryError::from(format!("Id '{id}' not found")))?;
                *id
            }
            QueryId::Alias(alias) => db_data
                .aliases
                .value(alias)?
                .ok_or(QueryError::from(format!("Alias '{alias}' not found")))?,
        })
    }

    fn graph_index_from_query_id(id: &QueryId, data: &DbData) -> Result<GraphIndex, QueryError> {
        Ok(GraphIndex::from(Self::get_index_from_id(id, data)?))
    }

    fn insert_alias(&self, data: InsertAlias, result: &mut QueryResult) -> Result<(), QueryError> {
        let mut mut_data = self.data.write()?;
        let index = Self::get_index_from_id(&data.id, &mut_data)?;
        mut_data.aliases.insert(&data.alias, &index)?;
        result.result += 1;
        Ok(())
    }

    fn insert_edge(
        &self,
        data: InsertEdge,
        context: &mut Context,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        let (from, to) = self.get_from_to(data, context)?;
        self.insert_edge_write_data(from, to, context)?;

        result.result += 1;
        result.elements.push(crate::DbElement {
            index: context.index,
            values: vec![],
        });

        Ok(())
    }

    fn insert_edge_write_data(
        &self,
        from: GraphIndex,
        to: GraphIndex,
        context: &mut Context,
    ) -> Result<(), QueryError> {
        let mut mut_data = self.data.write()?;
        let graph_index = mut_data.graph.insert_edge(&from, &to)?.index;
        mut_data.next_edge -= 1;
        Ok(mut_data.indexes.insert(&context.index, &graph_index)?)
    }

    fn insert_node(&self, data: InsertNode, result: &mut QueryResult) -> Result<(), QueryError> {
        let index = self.insert_node_write_data(data)?;

        result.result += 1;
        result.elements.push(crate::DbElement {
            index,
            values: vec![],
        });

        Ok(())
    }

    fn insert_node_write_data(&self, data: InsertNode) -> Result<i64, QueryError> {
        let mut mut_data = self.data.write()?;
        let index = mut_data.next_node;
        let graph_index = mut_data.graph.insert_node()?.index;
        mut_data.next_node += 1;

        if let Some(alias) = data.alias {
            mut_data.aliases.insert(&alias, &index)?
        }

        mut_data.indexes.insert(&index, &graph_index)?;

        Ok(index)
    }

    fn process_command(
        &self,
        command: Commands,
        context: &mut Context,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        match command {
            Commands::InsertAlias(data) => self.insert_alias(data, result),
            Commands::InsertEdge(data) => self.insert_edge(data, context, result),
            Commands::InsertNode(data) => self.insert_node(data, result),
            Commands::RemoveAlias(data) => self.remove_alias(data, result),
            Commands::RemoveEdge(data) => self.remove_edge(data, result),
            Commands::RemoveNode(data) => self.remove_node(data, result),
        }
    }

    fn remove_alias(&self, data: RemoveAlias, result: &mut QueryResult) -> Result<(), QueryError> {
        let mut db_data = self.data.write()?;
        db_data.aliases.remove_key(&data.alias)?;
        result.result += 1;
        Ok(())
    }

    fn remove_edge(&self, data: RemoveEdge, result: &mut QueryResult) -> Result<(), QueryError> {
        let mut db_data = self.data.write()?;
        let index = Self::graph_index_from_query_id(&data.id, &db_data)?;
        db_data.graph.remove_edge(&index)?;
        result.result += 1;
        Ok(())
    }

    fn remove_node(
        &self,
        data: crate::commands::remove_node::RemoveNode,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        let mut db_data = self.data.write()?;
        let index = Self::graph_index_from_query_id(&data.id, &db_data)?;
        db_data.graph.remove_node(&index)?;
        result.result += 1;
        Ok(())
    }
}
