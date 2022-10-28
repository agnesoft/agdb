use agdb_commands::CommandExecutor;
use agdb_commands::CommandStack;
use agdb_commands::Commands;
use agdb_db_error::DbError;

#[derive(Default)]
struct Executor {
    redo_result: Vec<Result<(), DbError>>,
    undo_result: Vec<Result<(), DbError>>,
}

impl CommandExecutor for Executor {
    fn redo(&mut self, _command: &mut Commands) -> Result<(), DbError> {
        self.redo_result.pop().unwrap()
    }

    fn undo(&mut self, _command: &mut Commands) -> Result<(), DbError> {
        self.undo_result.pop().unwrap()
    }
}

#[test]
fn clear() {
    let mut stack = CommandStack::new();
    stack.clear();
    stack.push(Commands::InsertNode);
    stack.clear();

    let mut executor = Executor {
        redo_result: vec![],
        undo_result: vec![],
    };

    assert_eq!(stack.redo(&mut executor), Ok(()));
}

#[test]
fn derived_from_default() {
    let mut _stack = CommandStack::default();
}

#[test]
fn empty() {
    let mut stack = CommandStack::new();

    let mut executor = Executor {
        redo_result: vec![],
        undo_result: vec![],
    };

    assert_eq!(stack.redo(&mut executor), Ok(()));
    assert_eq!(stack.undo(&mut executor), Ok(()));
}

#[test]
fn redo() {
    let mut stack = CommandStack::new();
    stack.push(Commands::InsertNode);
    stack.push(Commands::InsertNode);
    stack.push(Commands::InsertEdge);

    let mut executor = Executor {
        redo_result: vec![Ok(()), Ok(()), Ok(())],
        undo_result: vec![],
    };

    assert_eq!(stack.redo(&mut executor), Ok(()));
    assert_eq!(stack.redo(&mut executor), Ok(()));

    assert!(executor.redo_result.is_empty());
}

#[test]
fn undo() {
    let mut stack = CommandStack::new();
    stack.push(Commands::InsertNode);
    stack.push(Commands::InsertNode);
    stack.push(Commands::InsertEdge);

    let mut executor = Executor {
        redo_result: vec![Err(DbError::from("error")), Ok(()), Ok(())],
        undo_result: vec![Ok(()), Ok(())],
    };

    assert_eq!(stack.redo(&mut executor), Err(DbError::from("error")));
    assert_eq!(stack.undo(&mut executor), Ok(()));
    assert_eq!(stack.undo(&mut executor), Ok(()));

    assert!(executor.redo_result.is_empty());
    assert!(executor.undo_result.is_empty());
}

#[test]
fn undo_not_redone() {
    let mut stack = CommandStack::new();
    stack.push(Commands::InsertNode);
    stack.push(Commands::InsertNode);
    stack.push(Commands::InsertEdge);

    let mut executor = Executor {
        redo_result: vec![],
        undo_result: vec![],
    };

    assert_eq!(stack.undo(&mut executor), Ok(()));
}
