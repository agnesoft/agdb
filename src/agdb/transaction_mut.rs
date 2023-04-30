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

        for command in &mut self.commands {
            Self::redo_command(command, self.db, &mut context, &mut result)?;
        }

        Ok(result)
    }

    pub(crate) fn commit(self) -> Result<(), QueryError> {
        Ok(())
    }

    pub(crate) fn rollback(mut self) -> Result<(), QueryError> {
        for command in self.commands.iter_mut().rev() {
            Self::undo_command(command, self.db)?;
        }

        Ok(())
    }

    fn redo_command(
        command: &mut CommandsMut,
        db: &mut Db,
        context: &mut Context,
        result: &mut QueryResult,
    ) -> Result<(), QueryError> {
        match command {
            CommandsMut::InsertAlias(data) => data.redo(db, result, context),
            CommandsMut::InsertEdge(data) => data.redo(db, context),
            CommandsMut::InsertIndex(data) => data.redo(db, result, context),
            CommandsMut::InsertNode(data) => data.redo(db, context),
            CommandsMut::RemoveAlias(data) => data.redo(db, result, context),
            CommandsMut::RemoveEdge(data) => data.redo(db, context),
            CommandsMut::RemoveIndex(data) => data.redo(db, result, context),
            CommandsMut::RemoveNode(data) => data.redo(db, context),
        }
    }

    fn undo_command(command: &mut CommandsMut, db: &mut Db) -> Result<(), QueryError> {
        match command {
            CommandsMut::InsertAlias(data) => data.undo(db),
            CommandsMut::InsertEdge(data) => data.undo(db),
            CommandsMut::InsertIndex(data) => data.undo(db),
            CommandsMut::InsertNode(data) => data.undo(db),
            CommandsMut::RemoveAlias(data) => data.undo(db),
            CommandsMut::RemoveEdge(data) => data.undo(db),
            CommandsMut::RemoveIndex(data) => data.undo(db),
            CommandsMut::RemoveNode(data) => data.undo(db),
        }
    }
}
