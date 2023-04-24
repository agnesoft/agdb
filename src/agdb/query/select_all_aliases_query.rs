use super::Query;
use crate::commands::Commands;
use crate::QueryError;

pub struct SelectAllAliases {}

impl Query for SelectAllAliases {
    fn commands(&self) -> Result<Vec<Commands>, QueryError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn commands() {
        SelectAllAliases {}.commands().unwrap();
    }
}
