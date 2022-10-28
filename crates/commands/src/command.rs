use crate::Commands;

pub(crate) struct Command {
    pub(crate) commands: Commands,
    pub(crate) executed: bool,
}

impl Command {
    pub(crate) fn new(commands: Commands) -> Self {
        Self {
            commands,
            executed: false,
        }
    }
}
