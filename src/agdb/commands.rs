pub mod command_executor;
pub mod command_stack;

mod cmd;

#[allow(dead_code)]
pub enum Commands {
    InsertEdge,
    InsertNode,
}
