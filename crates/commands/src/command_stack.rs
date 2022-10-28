use crate::command::Command;
use crate::command_executor::CommandExecutor;
use crate::commands::Commands;
use agdb_db_error::DbError;

#[derive(Default)]
pub struct CommandStack {
    stack: Vec<Command>,
}

impl CommandStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.stack.clear();
    }

    pub fn push(&mut self, command: Commands) {
        self.stack.push(Command::new(command));
    }

    pub fn redo<Executor: CommandExecutor>(
        &mut self,
        executor: &mut Executor,
    ) -> Result<(), DbError> {
        for command in self.stack.iter_mut() {
            if !command.executed {
                executor.redo(&mut command.commands)?;
                command.executed = true;
            }
        }

        Ok(())
    }

    pub fn undo<Executor: CommandExecutor>(
        &mut self,
        executor: &mut Executor,
    ) -> Result<(), DbError> {
        for command in self.stack.iter_mut().rev() {
            if command.executed {
                executor.undo(&mut command.commands)?;
                command.executed = false;
            }
        }

        Ok(())
    }
}
