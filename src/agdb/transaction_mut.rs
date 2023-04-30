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
    commands: Vec<CommandsMut>,
}

impl<'a> TransactionMut<'a> {
    pub fn new(data: &'a mut Db) -> Self {
        Self {
            db: data,
            commands: vec![],
        }
    }

    pub fn exec<T: Query>(&self, query: &T) -> Result<QueryResult, QueryError> {
        Transaction::new(self.db).exec(query)
    }

    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError> {
        let mut context = Context {
            id: DbId(0),
            graph_index: GraphIndex { index: 0 },
        };

        let mut result = QueryResult {
            result: 0,
            elements: vec![],
        };

        self.commands = query.commands()?;

        for command in self.commands {
            self.redo_command(command, &mut context, &mut result)?;
        }

        Ok(result)
    }

    pub(crate) fn commit(self) -> Result<(), QueryError> {
        Ok(())
    }

    pub(crate) fn rollback(mut self) -> Result<(), QueryError> {
        let mut context = Context {
            id: DbId(0),
            graph_index: GraphIndex { index: 0 },
        };
        let mut result = QueryResult {
            result: 0,
            elements: vec![],
        };

        for command in self.commands {
            self.undo_command(command)?;
        }

        Ok(())
    }

    fn redo_command(
        &mut self,
        command: CommandsMut,
        context: &mut Context,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        match command {
            CommandsMut::InsertAlias(data) => data.redo(self.db, result, context),
            CommandsMut::InsertEdge(data) => data.redo(self.db, context),
            CommandsMut::InsertIndex(data) => data.redo(self.db, result, context),
            CommandsMut::InsertNode(data) => data.redo(self.db, context),
            CommandsMut::RemoveAlias(data) => data.redo(self.db, result, context),
            CommandsMut::RemoveEdge(data) => data.redo(self.db, context),
            CommandsMut::RemoveIndex(data) => data.redo(self.db, result, context),
            CommandsMut::RemoveNode(data) => data.redo(self.db, context),
            CommandsMut::None => Ok(()),
        }
    }

    fn undo_command(&mut self, command: CommandsMut) -> Result<(), QueryError> {
        match command {
            CommandsMut::InsertAlias(data) => data.undo(self.db),
            CommandsMut::InsertEdge(data) => data.undo(self.db),
            CommandsMut::InsertIndex(data) => data.undo(self.db),
            CommandsMut::InsertNode(data) => data.undo(self.db),
            CommandsMut::RemoveAlias(data) => data.undo(self.db),
            CommandsMut::RemoveEdge(data) => data.undo(self.db),
            CommandsMut::RemoveIndex(data) => data.undo(self.db),
            CommandsMut::RemoveNode(data) => data.undo(self.db),
            CommandsMut::None => Ok(()),
        }
    }
}
