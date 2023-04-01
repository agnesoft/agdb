use crate::commands_mut::CommandsMut;
use crate::db::db_context::Context;
use crate::query::Query;
use crate::query::QueryMut;
use crate::Db;
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
        Transaction::new(&self.db).exec(query)
    }

    pub fn exec_mut<T: QueryMut>(&mut self, query: &T) -> Result<QueryResult, QueryError> {
        let mut context = Context { index: 0 };
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
        let mut context = Context { index: 0 };
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
            CommandsMut::InsertAlias(data) => data.process(&mut self.db, result),
            CommandsMut::InsertEdge(data) => data.process(&mut self.db, context, result),
            CommandsMut::InsertNode(data) => data.process(&mut self.db, result),
            CommandsMut::RemoveAlias(data) => data.process(&mut self.db, result),
            CommandsMut::RemoveEdge(data) => data.process(&mut self.db, result),
            CommandsMut::RemoveNode(data) => data.process(&mut self.db, result),
        }
    }
}
