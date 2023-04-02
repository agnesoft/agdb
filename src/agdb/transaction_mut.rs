use crate::commands_mut::CommandsMut;
use crate::db::db_context::Context;
use crate::graph::graph_index::GraphIndex;
use crate::query::Query;
use crate::query::QueryMut;
use crate::Db;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;
use crate::Transaction;

pub struct TransactionMut<'a> {
    db: &'a mut Db,
    undo_stack: Vec<CommandsMut>,
}

impl<'a> TransactionMut<'a> {
    pub fn new(data: &'a mut Db) -> Self {
        Self {
            db: data,
            undo_stack: vec![],
        }
    }

    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        Transaction::new(self.db).exec(query)
    }

    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError> {
        let mut context = Context {
            id: DbId { id: 0 },
            graph_index: GraphIndex { index: 0 },
        };
        let mut result = QueryResult {
            result: 0,
            elements: vec![],
        };

        for command in query.commands()? {
            let undo = self.process_command(command, &mut context, &mut result)?;
            self.undo_stack.push(undo);
        }

        Ok(result)
    }

    pub(crate) fn commit(self) -> Result<(), QueryError> {
        Ok(())
    }

    pub(crate) fn rollback(mut self) -> Result<(), QueryError> {
        let mut context = Context {
            id: DbId { id: 0 },
            graph_index: GraphIndex { index: 0 },
        };
        let mut result = QueryResult {
            result: 0,
            elements: vec![],
        };
        let mut undo_commands = vec![];
        std::mem::swap(&mut self.undo_stack, &mut undo_commands);

        for command in undo_commands {
            self.process_command(command, &mut context, &mut result)?;
        }

        Ok(())
    }

    fn process_command(
        &mut self,
        command: CommandsMut,
        context: &mut Context,
        result: &mut QueryResult,
    ) -> Result<CommandsMut, QueryError> {
        match command {
            CommandsMut::InsertAlias(data) => data.process(self.db, context),
            CommandsMut::InsertAliasId(data) => data.process(self.db),
            CommandsMut::InsertEdge(data) => data.process(self.db, context),
            CommandsMut::InsertIndex(data) => data.process(self.db, result, context),
            CommandsMut::InsertIndexId(data) => data.process(self.db),
            CommandsMut::InsertNode(data) => data.process(self.db, context),
            CommandsMut::RemoveAlias(data) => data.process(self.db, context),
            CommandsMut::RemoveEdge(data) => data.process(self.db, context),
            CommandsMut::RemoveIndex(data) => data.process(self.db, result, context),
            CommandsMut::RemoveEdgeIndex(data) => data.process(self.db),
            CommandsMut::RemoveIndexId(data) => data.process(self.db, result, context),
            CommandsMut::RemoveNode(data) => data.process(self.db, context),
            CommandsMut::RemoveNodeIndex(data) => data.process(self.db),
        }
    }
}
