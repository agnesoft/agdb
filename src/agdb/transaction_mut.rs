use crate::commands_mut::insert_alias::InsertAlias;
use crate::commands_mut::insert_edge::InsertEdge;
use crate::commands_mut::insert_node::InsertNode;
use crate::commands_mut::remove_alias::RemoveAlias;
use crate::commands_mut::remove_edge::RemoveEdge;
use crate::commands_mut::remove_node::RemoveNode;
use crate::commands_mut::CommandsMut;
use crate::db::db_context::Context;
use crate::db::db_data;
use crate::db::db_data::DbData;
use crate::graph::graph_index::GraphIndex;
use crate::query::query_id::QueryId;
use crate::query::Query;
use crate::query::QueryMut;
use crate::DbElement;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;

pub struct TransactionMut<'a> {
    pub(crate) data: &'a mut DbData,
}

impl<'a> TransactionMut<'a> {
    pub fn new(data: &'a mut DbData) -> Self {
        Self { data }
    }

    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        Transaction { data: self.data }.exec(query)
    }

    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError> {
        let mut context = Context { index: 0 };
        let mut result = QueryResult {
            result: 0,
            elements: vec![],
        };

        for command in query.commands()? {
            self.process_command(command, &mut context, &mut result)?;
        }

        Ok(result)
    }

    pub(crate) fn commit(self) -> Result<(), QueryError> {
        Ok(())
    }

    pub(crate) fn rollback(self) -> Result<(), QueryError> {
        Ok(())
    }

    fn get_from_to(
        &self,
        data: InsertEdge,
        context: &mut Context,
    ) -> Result<(GraphIndex, GraphIndex), QueryError> {
        let from = graph_index_from_id(&data.from, self.data)?;
        let to = graph_index_from_id(&data.to, self.data)?;
        context.index = self.data.next_edge;

        Ok((from, to))
    }

    fn insert_alias(
        &mut self,
        data: InsertAlias,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        let index = db_data::index_from_id(&data.id, &self.data)?;
        self.data.aliases.insert(&data.alias, &index)?;
        result.result += 1;
        Ok(())
    }

    fn insert_edge(
        &mut self,
        data: InsertEdge,
        context: &mut Context,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        let (from, to) = self.get_from_to(data, context)?;
        self.insert_edge_write_data(from, to, context)?;

        result.result += 1;
        result.elements.push(DbElement {
            index: context.index,
            values: vec![],
        });

        Ok(())
    }

    fn insert_edge_write_data(
        &mut self,
        from: GraphIndex,
        to: GraphIndex,
        context: &mut Context,
    ) -> Result<(), QueryError> {
        let graph_index = self.data.graph.insert_edge(&from, &to)?.index;
        self.data.next_edge -= 1;
        Ok(self.data.indexes.insert(&context.index, &graph_index)?)
    }

    fn insert_node(
        &mut self,
        data: InsertNode,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        let index = self.insert_node_write_data(data)?;

        result.result += 1;
        result.elements.push(DbElement {
            index,
            values: vec![],
        });

        Ok(())
    }

    fn insert_node_write_data(&mut self, data: InsertNode) -> Result<i64, QueryError> {
        let index = self.data.next_node;
        let graph_index = self.data.graph.insert_node()?.index;
        self.data.next_node += 1;

        if let Some(alias) = data.alias {
            self.data.aliases.insert(&alias, &index)?
        }

        self.data.indexes.insert(&index, &graph_index)?;

        Ok(index)
    }

    fn process_command(
        &mut self,
        command: CommandsMut,
        context: &mut Context,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        match command {
            CommandsMut::InsertAlias(data) => self.insert_alias(data, result),
            CommandsMut::InsertEdge(data) => self.insert_edge(data, context, result),
            CommandsMut::InsertNode(data) => self.insert_node(data, result),
            CommandsMut::RemoveAlias(data) => self.remove_alias(data, result),
            CommandsMut::RemoveEdge(data) => self.remove_edge(data, result),
            CommandsMut::RemoveNode(data) => self.remove_node(data, result),
        }
    }

    fn remove_alias(
        &mut self,
        data: RemoveAlias,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        self.data.aliases.remove_key(&data.alias)?;
        result.result += 1;
        Ok(())
    }

    fn remove_edge(
        &mut self,
        data: RemoveEdge,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        let index = graph_index_from_id(&data.id, &self.data)?;
        self.data.graph.remove_edge(&index)?;
        result.result += 1;
        Ok(())
    }

    fn remove_node(
        &mut self,
        data: RemoveNode,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        let index = graph_index_from_id(&data.id, &self.data)?;
        self.data.graph.remove_node(&index)?;
        result.result += 1;
        Ok(())
    }
}

fn graph_index_from_id(id: &QueryId, db_data: &DbData) -> Result<GraphIndex, QueryError> {
    let graph_index = match id {
        QueryId::Id(id) => db_data
            .indexes
            .value(id)?
            .ok_or(QueryError::from(format!("Id '{id}' not found")))?,
        QueryId::Alias(alias) => {
            let id = db_data
                .aliases
                .value(alias)?
                .ok_or(QueryError::from(format!("Alias '{alias}' not found")))?;

            db_data.indexes.value(&id)?.unwrap_or_default()
        }
    };

    Ok(GraphIndex::from(graph_index))
}
