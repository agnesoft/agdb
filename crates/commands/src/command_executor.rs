use crate::Commands;
use agdb_db_error::DbError;

pub trait CommandExecutor {
    fn redo(&mut self, command: &mut Commands) -> Result<(), DbError>;
    fn undo(&mut self, command: &mut Commands) -> Result<(), DbError>;
}
